use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use service::{
    database::{Database, DatabaseError, DatabaseFuture, StoredEvent},
    event_store::{EventStore, StoreError},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::sync::Notify;
use tower::ServiceExt;

#[path = "../../service/tests/support/database_contract.rs"]
#[allow(dead_code)]
mod shared;

#[derive(Default)]
struct ControlledDatabase {
    memory: database_memory::MemoryDatabase,
    fail_read: AtomicBool,
    reads: AtomicUsize,
    corrupt_read: AtomicBool,
    fail_write: AtomicBool,
    commit_then_fail: AtomicBool,
    pause_write: AtomicBool,
    entered: Notify,
    release: Notify,
}
impl Database for ControlledDatabase {
    fn list(&self) -> DatabaseFuture<'_, Vec<String>> {
        self.memory.list()
    }
    fn read<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, Option<StoredEvent>> {
        Box::pin(async move {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if self.fail_read.load(Ordering::SeqCst) {
                return Err(DatabaseError::Unavailable("injected failure".into()));
            }
            if self.corrupt_read.load(Ordering::SeqCst) {
                return Err(DatabaseError::Corrupt("injected corrupt payload".into()));
            }
            self.memory.read(name).await
        })
    }
    fn create(&self, event: StoredEvent) -> DatabaseFuture<'_, ()> {
        self.memory.create(event)
    }
    fn replace(&self, event: StoredEvent) -> DatabaseFuture<'_, ()> {
        Box::pin(async move {
            if self.pause_write.swap(false, Ordering::SeqCst) {
                self.entered.notify_one();
                self.release.notified().await;
            }
            if self.commit_then_fail.load(Ordering::SeqCst) {
                self.memory.replace(event.clone()).await?;
            }
            if self.fail_write.load(Ordering::SeqCst) {
                return Err(DatabaseError::Unavailable("injected write failure".into()));
            }
            self.memory.replace(event).await
        })
    }
    fn delete<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, bool> {
        self.memory.delete(name)
    }
}
async fn setup() -> (Arc<ControlledDatabase>, EventStore) {
    let db = Arc::new(ControlledDatabase::default());
    let store = EventStore::new(db.clone(), "test-salt".into());
    store
        .create_event("e", shared::event("e").state)
        .await
        .unwrap();
    (db, store)
}

#[tokio::test]
async fn writes_publish_only_after_commit_and_survive_caller_cancellation() {
    let (db, store) = setup().await;
    let mut timer = store.subscribe_timer("e").await.unwrap().unwrap();
    db.pause_write.store(true, Ordering::SeqCst);
    let task_store = store.clone();
    let request = tokio::spawn(async move { task_store.patch_time("e", 42).await });
    db.entered.notified().await;
    assert!(matches!(
        timer.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    assert_eq!(
        db.memory
            .read("e")
            .await
            .unwrap()
            .unwrap()
            .state
            .time_seconds,
        -60
    );
    request.abort();
    db.release.notify_one();
    let update = tokio::time::timeout(std::time::Duration::from_secs(5), timer.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(update.current_time_seconds, 42);
    assert_eq!(
        store
            .current_timer("e")
            .await
            .unwrap()
            .unwrap()
            .current_time_seconds,
        42
    );
}

#[tokio::test]
async fn failed_write_does_not_publish_candidate_or_change_data() {
    let (db, store) = setup().await;
    let mut timer = store.subscribe_timer("e").await.unwrap().unwrap();
    db.fail_write.store(true, Ordering::SeqCst);
    assert!(matches!(
        store.patch_time("e", 42).await,
        Err(StoreError::ReportedStorage(_))
    ));
    assert_eq!(
        store
            .current_timer("e")
            .await
            .unwrap()
            .unwrap()
            .current_time_seconds,
        -60
    );
    assert!(matches!(
        timer.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Closed)
    ));
}

#[tokio::test]
async fn ambiguous_commit_reloads_authoritative_state_and_closes_old_streams() {
    let (db, store) = setup().await;
    let mut timer = store.subscribe_timer("e").await.unwrap().unwrap();
    db.commit_then_fail.store(true, Ordering::SeqCst);
    db.fail_write.store(true, Ordering::SeqCst);
    assert!(store.patch_time("e", 42).await.is_err());
    assert!(matches!(
        timer.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Closed)
    ));
    assert_eq!(
        store
            .current_timer("e")
            .await
            .unwrap()
            .unwrap()
            .current_time_seconds,
        42
    );
    db.fail_write.store(false, Ordering::SeqCst);
    db.commit_then_fail.store(false, Ordering::SeqCst);
    store.patch_time("e", 43).await.unwrap();
    assert_eq!(
        store
            .current_timer("e")
            .await
            .unwrap()
            .unwrap()
            .current_time_seconds,
        43
    );
}

#[tokio::test]
async fn storage_unavailability_is_503_not_404() {
    let (db, store) = setup().await;
    db.fail_read.store(true, Ordering::SeqCst);
    let app = server_v2::app(server_v2::AppState {
        store,
        public_url: "http://localhost".parse().unwrap(),
        internal_tokens: Arc::new(Default::default()),
    });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/events/e/contests")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json: serde_json::Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json["errors"][0]["code"], "storage_unavailable");
}

#[tokio::test]
async fn corrupt_storage_is_500() {
    let (db, store) = setup().await;
    db.corrupt_read.store(true, Ordering::SeqCst);
    let app = server_v2::app(server_v2::AppState {
        store,
        public_url: "http://localhost".parse().unwrap(),
        internal_tokens: Arc::new(Default::default()),
    });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/events/e/contests")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn sqlite_service_restart_restores_all_resources_and_revelation_urls() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("contest.sqlite3");
    let urls;
    let expected;
    {
        let database = Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap());
        let store = EventStore::new(database.clone(), "shared-salt".into());
        store
            .create_event("e", shared::event("e").state)
            .await
            .unwrap();
        store
            .create_contest(
                "e",
                "c",
                serde_json::from_value(
                    serde_json::json!({"name":"c","codes":["team"],"salt":"contest-salt"}),
                )
                .unwrap(),
            )
            .await
            .unwrap();
        store
            .create_site(
                "e",
                "c",
                "s",
                serde_json::from_value(
                    serde_json::json!({"name":"s","codes":["team"],"salt":"site-salt"}),
                )
                .unwrap(),
            )
            .await
            .unwrap();
        store.add_runs("e", vec![serde_json::from_value(serde_json::json!({"id":1,"team_login":"team1","prob":"A","time_seconds":10,"answer":"Y"})).unwrap()]).await.unwrap();
        store.patch_time("e", 20).await.unwrap();
        urls = store
            .revelation_urls("e", &"https://example.com".parse().unwrap())
            .await
            .unwrap()
            .unwrap();
        expected = database.read("e").await.unwrap();
    }
    let database = Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap());
    assert_eq!(database.read("e").await.unwrap(), expected);
    let store = EventStore::new(database, "shared-salt".into());
    assert_eq!(
        serde_json::to_value(
            store
                .revelation_urls("e", &"https://example.com".parse().unwrap())
                .await
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(urls).unwrap()
    );
    assert_eq!(
        store
            .current_timer("e")
            .await
            .unwrap()
            .unwrap()
            .current_time_seconds,
        20
    );
    let mut runs = store.subscribe_runs("e").await.unwrap().unwrap();
    assert_eq!(runs.recv().await.unwrap().id, 1);
}

#[test]
fn sentry_reports_detached_failure_once_with_request_context() {
    use sentry::SentryFutureExt;
    use tracing::Instrument;
    use tracing_subscriber::prelude::*;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    tracing_subscriber::registry()
        .with(sentry_tracing::layer())
        .init();
    let events = sentry::test::with_captured_events(|| {
        runtime.block_on(async {
            let (db, store) = setup().await;
            sentry::configure_scope(|scope| {
                scope.set_tag("request_id", "cancelled-request");
                scope.set_user(Some(sentry::User {
                    username: Some("admin".into()),
                    ..Default::default()
                }));
            });
            db.pause_write.store(true, Ordering::SeqCst);
            db.fail_write.store(true, Ordering::SeqCst);
            let barrier = store.clone();
            let request = tokio::spawn(
                async move { store.patch_time("e", 42).await }
                    .instrument(tracing::info_span!(
                        "request",
                        request_id = "cancelled-request"
                    ))
                    .bind_hub(sentry::Hub::current()),
            );
            db.entered.notified().await;
            request.abort();
            let _ = request.await;
            db.release.notify_one();
            barrier.current_timer("e").await.unwrap();
        })
    });
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].tags["request_id"], "cancelled-request");
    assert_eq!(
        events[0].user.as_ref().unwrap().username.as_deref(),
        Some("admin")
    );
    assert!(events[0].exception.iter().any(|e| {
        e.value
            .as_deref()
            .is_some_and(|s| s.contains("injected write failure"))
    }));
}

#[tokio::test]
async fn public_snapshots_do_not_reload_storage_and_observe_commits() {
    let (db, store) = setup().await;
    store.initialize().await.unwrap();
    let before = db.reads.load(Ordering::SeqCst);
    // Even a storage outage cannot turn public replay into repeated database work.
    db.fail_read.store(true, Ordering::SeqCst);
    for _ in 0..100 {
        assert!(store.current_timer("e").await.unwrap().is_some());
        assert!(store.subscribe_runs("e").await.unwrap().is_some());
        assert!(store.list_contests("missing").await.unwrap().is_none());
        assert_eq!(store.list_events().await.unwrap(), vec!["e"]);
    }
    assert_eq!(before, db.reads.load(Ordering::SeqCst));
    db.fail_read.store(false, Ordering::SeqCst);
    store.patch_time("e", 42).await.unwrap();
    assert_eq!(
        store
            .current_timer("e")
            .await
            .unwrap()
            .unwrap()
            .current_time_seconds,
        42
    );
    store.delete_event("e").await.unwrap();
    assert!(store.current_timer("e").await.unwrap().is_none());
}

#[tokio::test]
async fn sqlite_remote_credentials_survive_restart_and_revocation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("remote.sqlite3");
    let key;
    let expiry;
    {
        let database = Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap());
        let store = EventStore::new(database.clone(), "salt".into());
        store
            .create_event("e", shared::event("e").state)
            .await
            .unwrap();
        store
            .create_contest(
                "e",
                "c",
                serde_json::from_value(serde_json::json!({"name":"c","codes":[]})).unwrap(),
            )
            .await
            .unwrap();
        let before = service::remote_control::unix_seconds();
        let info = store.create_remote_session("e", "c").await.unwrap();
        assert!(info.expires_at >= before + 7 * 86400);
        key = info.key;
        expiry = info.expires_at;
        let stored = database.read("e").await.unwrap().unwrap();
        let records = &stored.contests["c"].remote_control;
        assert_eq!(
            records[&service::remote_control::remote_key_hash(&key)].expires_at,
            expiry
        );
        assert!(
            !serde_json::to_string(&stored).unwrap().contains(&key),
            "plaintext key must not be persisted"
        );
        let connection = store
            .remote_control_sender("e", "c", &key)
            .await
            .unwrap()
            .unwrap();
        store.patch_time("e", 42).await.unwrap();
        assert!(
            !connection.revoked.has_changed().unwrap(),
            "unrelated writes must preserve live sessions"
        );
    }
    {
        let database = Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap());
        let store = EventStore::new(database.clone(), "salt".into());
        let connection = store
            .remote_control_sender("e", "c", &key)
            .await
            .unwrap()
            .unwrap();
        assert!(!connection.revoked.has_changed().unwrap());
        let records = database
            .read("e")
            .await
            .unwrap()
            .unwrap()
            .contests
            .remove("c")
            .unwrap()
            .remote_control;
        assert_eq!(
            records[&service::remote_control::remote_key_hash(&key)].expires_at,
            expiry
        );
        store.revoke_remote_session("e", "c", &key).await.unwrap();
        assert!(*connection.revoked.borrow());
        assert!(
            store
                .remote_control_sender("e", "c", &key)
                .await
                .unwrap()
                .is_none()
        );
    }
    {
        let database = Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap());
        let store = EventStore::new(database, "salt".into());
        assert!(
            store
                .remote_control_sender("e", "c", &key)
                .await
                .unwrap()
                .is_none(),
            "revocation must survive restart"
        );
    }
}

#[tokio::test]
async fn expired_sqlite_credentials_are_not_restored() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("expiry.sqlite3");
    let key;
    {
        let database = Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap());
        let store = EventStore::new(database.clone(), "salt".into());
        store
            .create_event("e", shared::event("e").state)
            .await
            .unwrap();
        store
            .create_contest(
                "e",
                "c",
                serde_json::from_value(serde_json::json!({"name":"c","codes":[]})).unwrap(),
            )
            .await
            .unwrap();
        key = store.create_remote_session("e", "c").await.unwrap().key;
        drop(store);
        let mut event = database.read("e").await.unwrap().unwrap();
        for record in event
            .contests
            .get_mut("c")
            .unwrap()
            .remote_control
            .values_mut()
        {
            record.expires_at = service::remote_control::unix_seconds() - 1;
        }
        database.replace(event).await.unwrap();
    }
    let store = EventStore::new(
        Arc::new(database_sqlite::SqliteDatabase::open(&path).unwrap()),
        "salt".into(),
    );
    assert!(
        store
            .remote_control_sender("e", "c", &key)
            .await
            .unwrap()
            .is_none()
    );
    // Expired credentials do not consume registry slots.
    for _ in 0..16 {
        store.create_remote_session("e", "c").await.unwrap();
    }
}

#[tokio::test]
async fn durable_credentials_obey_global_limit_across_events() {
    let store = EventStore::new(
        Arc::new(database_memory::MemoryDatabase::new()),
        "salt".into(),
    );
    let mut first = None;
    for i in 0..17 {
        let event = format!("event-{i}");
        store
            .create_event(&event, shared::event(&event).state)
            .await
            .unwrap();
        store
            .create_contest(
                &event,
                "c",
                serde_json::from_value(serde_json::json!({"name":"c","codes":[]})).unwrap(),
            )
            .await
            .unwrap();
        if i < 16 {
            for _ in 0..16 {
                let key = store.create_remote_session(&event, "c").await.unwrap().key;
                if first.is_none() {
                    first = Some(key);
                }
            }
        }
    }
    assert!(matches!(
        store.create_remote_session("event-16", "c").await,
        Err(StoreError::LimitExceeded)
    ));
    store
        .revoke_remote_session("event-0", "c", &first.unwrap())
        .await
        .unwrap();
    store.create_remote_session("event-16", "c").await.unwrap();
}
