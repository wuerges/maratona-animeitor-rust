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
    atomic::{AtomicBool, Ordering},
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
        Err(StoreError::Storage(_))
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
