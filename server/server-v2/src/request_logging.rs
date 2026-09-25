use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Request},
    middleware::{self, Next},
    response::Response,
};
use std::time::Instant;
use tracing::Instrument;

#[derive(Clone)]
pub(crate) struct RequestSpan(pub tracing::Span);

pub(crate) fn layer(router: Router) -> Router {
    router.layer(middleware::from_fn(log_request))
}

async fn log_request(mut request: Request<Body>, next: Next) -> Response {
    let header = request
        .headers()
        .get("x-request-id")
        .cloned()
        .unwrap_or_else(|| {
            HeaderValue::from_str(&nanoid::nanoid!()).expect("Nano ID is a valid header")
        });
    request.headers_mut().insert("x-request-id", header.clone());
    let span = tracing::info_span!(
        "http_request",
        request_id = ?header,
        method = %request.method(),
        path = %request.uri().path(),
        username = tracing::field::Empty,
    );
    request.extensions_mut().insert(RequestSpan(span.clone()));
    async move {
        let start = Instant::now();
        tracing::info!("request started");
        let mut response = next.run(request).await;
        response.headers_mut().insert("x-request-id", header);
        tracing::info!(
            status = response.status().as_u16(),
            duration_ms = start.elapsed().as_secs_f64() * 1000.0,
            "request completed"
        );
        response
    }
    .instrument(span)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };
    use tower::ServiceExt;

    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn logs_requests_with_verified_login_and_no_secrets() {
        let output = Capture(Arc::default());
        let writer = output.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        let app = crate::app(crate::AppState {
            public_url: "https://example.com".parse().unwrap(),
            store: service::event_store::EventStore::new(
                Arc::new(database_memory::MemoryDatabase::new()),
                "salt".into(),
            ),
            internal_tokens: Arc::new([("admin".into(), "secret-token".into())].into()),
        });
        let credentials = base64::engine::general_purpose::STANDARD.encode("admin:secret-token");
        let mut ids = Vec::new();
        for (path, auth, status) in [
            (
                "/internal/events?secret=query-secret",
                Some(format!("Basic {credentials}")),
                200,
            ),
            ("/internal/events", None, 401),
            ("/missing", None, 404),
        ] {
            let mut request = Request::builder().uri(path);
            if let Some(auth) = auth {
                request = request.header("authorization", auth);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status().as_u16(), status);
            let id = response.headers()["x-request-id"]
                .to_str()
                .unwrap()
                .to_owned();
            assert_eq!(id.len(), 21);
            assert!(!ids.contains(&id));
            ids.push(id);
        }
        let logs = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        for id in ids {
            assert!(logs.contains(&id));
        }
        assert!(logs.contains("username=\"admin\""));
        assert!(logs.contains("status=401"));
        assert!(logs.contains("status=404"));
        for secret in ["secret-token", "query-secret", &credentials] {
            assert!(!logs.contains(secret));
        }
    }

    #[tokio::test]
    async fn attaches_id_to_request_and_http_rejections() {
        let app = layer(
            Router::new()
                .route(
                    "/echo",
                    axum::routing::get(|request: Request<Body>| async move {
                        request.headers()["x-request-id"]
                            .to_str()
                            .unwrap()
                            .to_owned()
                    }),
                )
                .layer(middleware::from_fn(crate::reject_internal_over_http)),
        );
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/echo").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let id = response.headers()["x-request-id"].clone();
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), id.as_bytes());
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/echo")
                    .header("x-request-id", "caller-operation-456")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()["x-request-id"], "caller-operation-456");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"caller-operation-456");
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/internal/events")
                    .header("x-request-id", "admin-operation-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::UPGRADE_REQUIRED);
        assert_eq!(response.headers()["x-request-id"], "admin-operation-123");
    }
}
