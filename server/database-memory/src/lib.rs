//! Isolated, process-local implementation of the service persistence contract.
use service::database::{Database, DatabaseError, DatabaseFuture, StoredEvent};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct MemoryDatabase {
    events: RwLock<Vec<StoredEvent>>,
}
impl MemoryDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Database for MemoryDatabase {
    fn list(&self) -> DatabaseFuture<'_, Vec<String>> {
        Box::pin(async {
            Ok(self
                .events
                .read()
                .await
                .iter()
                .map(|e| e.state.name.clone())
                .collect())
        })
    }
    fn read<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, Option<StoredEvent>> {
        Box::pin(async move {
            Ok(self
                .events
                .read()
                .await
                .iter()
                .find(|e| e.state.name == name)
                .cloned())
        })
    }
    fn create(&self, event: StoredEvent) -> DatabaseFuture<'_, ()> {
        Box::pin(async move {
            let mut events = self.events.write().await;
            if events.iter().any(|e| e.state.name == event.state.name) {
                return Err(DatabaseError::AlreadyExists);
            }
            events.push(event);
            Ok(())
        })
    }
    fn replace(&self, event: StoredEvent) -> DatabaseFuture<'_, ()> {
        Box::pin(async move {
            let mut events = self.events.write().await;
            let entry = events
                .iter_mut()
                .find(|e| e.state.name == event.state.name)
                .ok_or(DatabaseError::NotFound)?;
            *entry = event;
            Ok(())
        })
    }
    fn delete<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, bool> {
        Box::pin(async move {
            let mut events = self.events.write().await;
            let before = events.len();
            events.retain(|e| e.state.name != name);
            Ok(events.len() != before)
        })
    }
}
