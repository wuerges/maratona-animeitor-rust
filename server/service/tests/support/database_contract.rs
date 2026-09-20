use service::database::{Database, DatabaseError, StoredEvent};
use std::sync::Arc;

pub fn event(name: &str) -> StoredEvent {
    StoredEvent {
        state: serde_json::from_value(serde_json::json!({
            "name": name, "problems": ["A"],
            "teams": [{"login":"team1","nome":"Team","escola":"School"}],
            "score_freeze_time_seconds": 240, "penalty_seconds":1200, "time_seconds":-60,
            "salt":"event-salt"
        }))
        .unwrap(),
        contests: Default::default(),
        runs: vec![],
    }
}

/// The same externally observable contract runs against both implementations.
pub async fn contract(db: Arc<dyn Database>) {
    assert!(db.list().await.unwrap().is_empty());
    assert_eq!(db.read("missing").await.unwrap(), None);
    assert!(!db.delete("missing").await.unwrap());
    assert_eq!(
        db.replace(event("missing")).await,
        Err(DatabaseError::NotFound)
    );
    let first = event("z-last-alphabetically");
    let second = event("a-first-alphabetically");
    db.create(first.clone()).await.unwrap();
    db.create(second.clone()).await.unwrap();
    assert_eq!(
        db.list().await.unwrap(),
        [first.state.name.clone(), second.state.name.clone()]
    );
    assert_eq!(
        db.create(first.clone()).await,
        Err(DatabaseError::AlreadyExists)
    );
    let mut next = first.clone();
    next.state.time_seconds = 15;
    next.runs.push(serde_json::from_value(serde_json::json!({"id":1,"team_login":"team1","prob":"A","time_seconds":10,"answer":"Y"})).unwrap());
    db.replace(next.clone()).await.unwrap();
    assert_eq!(
        db.read(&first.state.name).await.unwrap(),
        Some(next.clone())
    );
    assert_eq!(
        db.read(&second.state.name).await.unwrap(),
        Some(second.clone())
    );
    next.runs[0].answer = service::event_store::Answer::No;
    db.replace(next.clone()).await.unwrap();
    assert_eq!(db.read(&first.state.name).await.unwrap(), Some(next));
    assert!(db.delete(&first.state.name).await.unwrap());
    db.create(first.clone()).await.unwrap();
    assert_eq!(
        db.read(&first.state.name).await.unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        db.list().await.unwrap(),
        [second.state.name, first.state.name]
    );
    let (left, right) = tokio::join!(db.create(event("race")), db.create(event("race")));
    assert_eq!(usize::from(left.is_ok()) + usize::from(right.is_ok()), 1);
    assert_eq!(
        left.err().or(right.err()),
        Some(DatabaseError::AlreadyExists)
    );
}
