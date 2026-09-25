use axum::{body::Body, http::Request};
use base64::Engine;
use sentry::SentryFutureExt;
use service::database::{Database, DatabaseError, DatabaseFuture, StoredEvent};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tower::ServiceExt;
use tracing_subscriber::prelude::*;

#[derive(Default)]
struct FailingDatabase {
    panic: AtomicBool,
}
impl Database for FailingDatabase {
    fn list(&self) -> DatabaseFuture<'_, Vec<String>> {
        Box::pin(async {
            tokio::task::yield_now().await;
            assert!(!self.panic.load(Ordering::SeqCst), "database panic test");
            Err(DatabaseError::Unavailable("database offline".into()))
        })
    }
    fn read<'a>(&'a self, _: &'a str) -> DatabaseFuture<'a, Option<StoredEvent>> {
        unreachable!()
    }
    fn create(&self, _: StoredEvent) -> DatabaseFuture<'_, ()> {
        unreachable!()
    }
    fn replace(&self, _: StoredEvent) -> DatabaseFuture<'_, ()> {
        unreachable!()
    }
    fn delete<'a>(&'a self, _: &'a str) -> DatabaseFuture<'a, bool> {
        unreachable!()
    }
}
fn request(user: &str) -> Request<Body> {
    Request::builder()
        .uri("/internal/events")
        .header("x-request-id", user)
        .header(
            "authorization",
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(format!("{user}:token"))
            ),
        )
        .body(Body::empty())
        .unwrap()
}

#[test]
fn errors_and_panics_keep_isolated_request_identity_across_awaits() {
    let _subscriber = tracing_subscriber::registry()
        .with(sentry_tracing::layer())
        .set_default();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let db = Arc::new(FailingDatabase::default());
    let app = server_v2::app(server_v2::AppState {
        public_url: "https://example.com".parse().unwrap(),
        store: service::event_store::EventStore::new(db.clone(), "salt".into()),
        internal_tokens: Arc::new(
            [
                ("alice".into(), "token".into()),
                ("bob".into(), "token".into()),
            ]
            .into(),
        ),
    });
    let events = sentry::test::with_captured_events_options(
        || {
            runtime.block_on(async {
                let (a, b) = tokio::join!(
                    app.clone().oneshot(request("alice")),
                    app.clone().oneshot(request("bob"))
                );
                assert_eq!(a.unwrap().status(), 503);
                assert_eq!(b.unwrap().status(), 503);
                db.panic.store(true, Ordering::SeqCst);
                let panic = tokio::spawn(
                    app.oneshot(request("alice"))
                        .bind_hub(sentry::Hub::current()),
                )
                .await;
                assert!(panic.unwrap_err().is_panic());
            })
        },
        sentry::apply_defaults(sentry::ClientOptions::default()),
    );
    assert_eq!(events.len(), 3);
    for event in &events {
        let username = event.user.as_ref().unwrap().username.as_deref().unwrap();
        assert_eq!(event.tags["request_id"], username);
        assert!(!event.exception.is_empty());
    }
    assert_eq!(
        events
            .iter()
            .filter(|e| e.user.as_ref().unwrap().username.as_deref() == Some("bob"))
            .count(),
        1
    );
    assert_eq!(events[2].level, sentry::Level::Fatal);
}
