#[path = "../../service/tests/support/database_contract.rs"]
mod shared;
use database_sqlite::SqliteDatabase;
use service::database::{Database, DatabaseError};
use std::sync::Arc;

#[tokio::test]
async fn sqlite_contract() {
    let dir = tempfile::tempdir().unwrap();
    shared::contract(Arc::new(
        SqliteDatabase::open(dir.path().join("test.sqlite3")).unwrap(),
    ))
    .await;
}

#[tokio::test]
async fn reopen_preserves_data_and_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    let mut event = shared::event("z");
    event.runs.push(serde_json::from_value(serde_json::json!({"id":7,"team_login":"team1","prob":"A","time_seconds":10,"answer":"Y"})).unwrap());
    {
        let db = SqliteDatabase::open(&path).unwrap();
        db.create(event.clone()).await.unwrap();
        db.create(shared::event("a")).await.unwrap();
    }
    let db = SqliteDatabase::open(&path).unwrap();
    assert_eq!(db.read("z").await.unwrap(), Some(event));
    assert_eq!(db.list().await.unwrap(), ["z", "a"]);
}

#[test]
fn refuses_newer_or_unrecognized_schema_and_invalid_paths() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA user_version=999;")
        .unwrap();
    assert!(matches!(
        SqliteDatabase::open(&path),
        Err(DatabaseError::Corrupt(_))
    ));
    connection
        .execute_batch("PRAGMA user_version=0; CREATE TABLE unrelated(x);")
        .unwrap();
    assert!(matches!(
        SqliteDatabase::open(&path),
        Err(DatabaseError::Corrupt(_))
    ));
    assert!(matches!(
        SqliteDatabase::open(dir.path().join("missing/db.sqlite3")),
        Err(DatabaseError::Unavailable(_))
    ));
}

#[tokio::test]
async fn timer_updates_do_not_rewrite_runs_and_failed_batches_roll_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    let db = SqliteDatabase::open(&path).unwrap();
    let mut event = shared::event("e");
    event.runs.push(serde_json::from_value(serde_json::json!({"id":1,"team_login":"team1","prob":"A","time_seconds":10,"answer":"Y"})).unwrap());
    db.create(event.clone()).await.unwrap();
    let inspection = rusqlite::Connection::open(&path).unwrap();
    inspection.execute_batch("CREATE TRIGGER reject_run_update BEFORE UPDATE ON runs BEGIN SELECT RAISE(ABORT, 'injected write failure'); END;").unwrap();
    event.state.time_seconds = 12;
    db.replace(event.clone()).await.unwrap();
    let before = event.clone();
    event.state.time_seconds = 99;
    event.runs[0].answer = service::event_store::Answer::No;
    assert!(db.replace(event).await.is_err());
    assert_eq!(db.read("e").await.unwrap(), Some(before));
    assert_eq!(
        inspection
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}
