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

impl StoredEvent {
    /// Structural storage invariants; roster/reference policy belongs to the service.
    pub fn normalize(&mut self) -> Result<(), DatabaseError> {
        let mut ids = std::collections::HashSet::new();
        if self.runs.iter().any(|run| !ids.insert(run.id)) {
            return Err(DatabaseError::Corrupt("duplicate run IDs".into()));
        }
        if self.contests.iter().any(|(name, contest)| {
            name != &contest.config.name
                || contest.sites.iter().any(|(name, site)| name != &site.name)
        }) {
            return Err(DatabaseError::Corrupt(
                "resource identifier mismatch".into(),
            ));
        }
        self.runs.sort_by_key(|run| (run.time_seconds, run.id));
        Ok(())
    }
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

/// Backend selection shared by the server and HTTP-only administration tools.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatabaseConfig {
    Memory {},
    Sqlite { path: std::path::PathBuf },
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::Memory {}
    }
}

/// Common storage instrumentation independent of the selected backend.
pub(crate) async fn observe<T>(
    operation: &'static str,
    future: DatabaseFuture<'_, T>,
) -> Result<T, DatabaseError> {
    let start = std::time::Instant::now();
    let result = future.await;
    metrics::histogram!("database_operation_seconds", "operation" => operation)
        .record(start.elapsed().as_secs_f64());
    if result.is_err() {
        metrics::counter!("database_errors_total", "operation" => operation).increment(1);
    }
    result
}
