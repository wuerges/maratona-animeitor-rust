//! Process-wide admission limits. Per-source limits belong at the trusted proxy.
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
    match tokio::time::timeout(Duration::from_secs(timeout_seconds), next.run(request)).await {
        Ok(response) => response,
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

pub fn upgrade<F, Fut>(ws: WebSocketUpgrade, permit: OwnedSemaphorePermit, callback: F) -> Response
where
    F: FnOnce(WebSocket) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    websocket(ws).on_upgrade(move |socket| async move {
        let _permit = permit;
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
