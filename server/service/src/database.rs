//! Persistence contract. Implementations store data; the service owns domain rules.
use data::event::{ContestConfig, EventState, Run, SiteConfig};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future, pin::Pin};

pub type DatabaseFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, DatabaseError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DatabaseError {
    #[error("storage unavailable: {0}")]
    Unavailable(String),
    #[error("invalid stored data: {0}")]
    Corrupt(String),
    #[error("event already exists")]
    AlreadyExists,
    #[error("event does not exist")]
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredContest {
    pub config: ContestConfig,
    pub sites: BTreeMap<String, SiteConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredEvent {
    pub state: EventState,
    pub contests: BTreeMap<String, StoredContest>,
    pub runs: Vec<Run>,
}

/// One service process owns a database. All methods are atomic. Listing preserves
/// creation order; replacing does not move an event, recreating appends it.
/// Implementations must return errors, never substitute empty/missing data.
pub trait Database: Send + Sync {
    fn list(&self) -> DatabaseFuture<'_, Vec<String>>;
    fn read<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, Option<StoredEvent>>;
    fn create(&self, event: StoredEvent) -> DatabaseFuture<'_, ()>;
    fn replace(&self, event: StoredEvent) -> DatabaseFuture<'_, ()>;
    fn delete<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, bool>;
}
