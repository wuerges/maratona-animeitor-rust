//! Process admission and response lifetimes; transport.rs bounds direct socket IO.
use axum::{
    Router,
    body::Body,
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub fn http(
    router: Router<crate::AppState>,
    capacity: usize,
    timeout_seconds: u64,
) -> Router<crate::AppState> {
    router.layer(middleware::from_fn_with_state(
        (Arc::new(Semaphore::new(capacity)), timeout_seconds),
        admit,
    ))
}
async fn admit(
    State((slots, timeout_seconds)): State<(Arc<Semaphore>, u64)>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let Ok(_permit) = slots.try_acquire_owned() else {
        service::resource_metrics::event("request_rejected");
        let mut response = crate::envelope::error_json(
            StatusCode::TOO_MANY_REQUESTS,
            "resource_limit",
            "request capacity exceeded",
        );
        response.headers_mut().insert(
            axum::http::header::RETRY_AFTER,
            axum::http::HeaderValue::from_static("1"),
        );
        return response;
    };
    let _active = service::resource_metrics::Active::new("http_handler");
    let writing = matches!(
        *request.method(),
        axum::http::Method::POST
            | axum::http::Method::PUT
            | axum::http::Method::PATCH
            | axum::http::Method::DELETE
    );
    match tokio::time::timeout(Duration::from_secs(timeout_seconds), next.run(request)).await {
        Ok(response) => response,
        Err(_) if writing => {
            service::resource_metrics::event("write_outcome_unknown");
            crate::envelope::error_json(
                StatusCode::SERVICE_UNAVAILABLE,
                "operation_outcome_unknown",
                "request timed out; the operation may still commit; reconcile state before retrying",
            )
        }
        Err(_) => crate::envelope::error_json(
            StatusCode::REQUEST_TIMEOUT,
            "request_timeout",
            format!("request exceeded {timeout_seconds} seconds"),
        ),
    }
}

pub fn websocket(ws: WebSocketUpgrade) -> WebSocketUpgrade {
    ws.max_message_size(4096)
        .max_frame_size(4096)
        .max_write_buffer_size(128 * 1024 + 8192)
}

pub fn upgrade<F, Fut>(
    kind: &'static str,
    ws: WebSocketUpgrade,
    permit: OwnedSemaphorePermit,
    callback: F,
) -> Response
where
    F: FnOnce(WebSocket) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    websocket(ws).on_upgrade(move |socket| async move {
        let _permit = permit;
        let _active = service::resource_metrics::Active::new(kind);
        callback(socket).await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn overload_is_rejected_without_waiting() {
        let slots = Arc::new(Semaphore::new(1));
        let permit = slots.clone().acquire_owned().await.unwrap();
        let app = Router::new()
            .route("/", axum::routing::get(|| async { StatusCode::OK }))
            .layer(middleware::from_fn_with_state((slots, 15), admit));
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        drop(permit);
        assert_eq!(
            app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }
    #[tokio::test(start_paused = true)]
    async fn stalled_request_hits_deadline_and_releases_capacity() {
        let slots = Arc::new(Semaphore::new(1));
        let app = Router::new()
            .route(
                "/",
                axum::routing::get(|| async { std::future::pending::<StatusCode>().await }),
            )
            .layer(middleware::from_fn_with_state((slots.clone(), 15), admit));
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(slots.available_permits(), 1);
    }
}

/// Ordinary responses retain admission until their body finishes or is dropped.
pub fn responses(router: Router, capacity: usize) -> Router {
    router.layer(middleware::from_fn_with_state(
        Arc::new(Semaphore::new(capacity)),
        admit_response,
    ))
}
async fn admit_response(
    State(slots): State<Arc<Semaphore>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Admission must precede handlers: rejecting a response after a mutation
    // committed would invite an unsafe retry of a successful operation.
    let Ok(permit) = slots.try_acquire_owned() else {
        service::resource_metrics::event("response_rejected");
        return crate::envelope::error_json(
            StatusCode::TOO_MANY_REQUESTS,
            "resource_limit",
            "response capacity exceeded",
        );
    };
    let connect = request.method() == axum::http::Method::CONNECT;
    let response = next.run(request).await;
    if response.status() == StatusCode::SWITCHING_PROTOCOLS
        || (connect && response.status().is_success())
    {
        return response;
    }
    let (parts, body) = response.into_parts();
    Response::from_parts(
        parts,
        Body::new(AdmittedBody {
            body,
            permit: Some(permit),
            active: Some(service::resource_metrics::Active::new("response_body")),
        }),
    )
}
struct AdmittedBody {
    active: Option<service::resource_metrics::Active>,
    body: Body,
    permit: Option<OwnedSemaphorePermit>,
}
impl http_body::Body for AdmittedBody {
    type Data = axum::body::Bytes;
    type Error = axum::Error;
    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        let result = std::pin::Pin::new(&mut this.body).poll_frame(cx);
        if matches!(result, std::task::Poll::Ready(None | Some(Err(_))))
            || this.body.is_end_stream()
        {
            this.permit.take();
            this.active.take();
        }
        result
    }
    fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }
    fn size_hint(&self) -> http_body::SizeHint {
        self.body.size_hint()
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn response_body_holds_slot_until_drop_or_consumption() {
        use http_body_util::BodyExt;
        let app = responses(
            Router::new().route("/asset", axum::routing::get(|| async { "static bytes" })),
            1,
        );
        let request = || {
            Request::builder()
                .uri("/asset")
                .body(Body::empty())
                .unwrap()
        };
        let first = app.clone().oneshot(request()).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(
            app.clone().oneshot(request()).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        drop(first);
        let next = app.clone().oneshot(request()).await.unwrap();
        assert_eq!(next.status(), StatusCode::OK);
        assert!(
            !next
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
        assert_eq!(
            app.oneshot(request()).await.unwrap().status(),
            StatusCode::OK
        );
    }
}
