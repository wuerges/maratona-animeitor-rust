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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StoredEvent {
    pub state: EventState,
    pub contests: BTreeMap<String, StoredContest>,
    pub runs: Vec<Run>,
}

impl<'de> Deserialize<'de> for StoredEvent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Before media became event-scoped, templates lived in each contest.
        // Explicit event values (including null) always win over legacy values.
        let mut value = serde_json::Value::deserialize(deserializer)?;
        for field in ["photo_url_format", "sound_url_format"] {
            let mut legacy = None;
            let mut conflicting = false;
            if let Some(contests) = value
                .get_mut("contests")
                .and_then(serde_json::Value::as_object_mut)
            {
                for contest in contests.values_mut() {
                    if let Some(old) = contest
                        .get_mut("config")
                        .and_then(serde_json::Value::as_object_mut)
                        .and_then(|c| c.remove(field))
                    {
                        if !old.is_null() {
                            match &legacy {
                                Some(previous) if previous != &old => {
                                    conflicting = true;
                                }
                                None => legacy = Some(old),
                                _ => {}
                            }
                        }
                    }
                }
            }
            if value["state"].get(field).is_none() {
                if conflicting {
                    return Err(serde::de::Error::custom(format!(
                        "conflicting legacy {field} values; configure one event-level value"
                    )));
                }
                if let Some(old) = legacy {
                    value["state"]
                        .as_object_mut()
                        .ok_or_else(|| serde::de::Error::custom("invalid event state"))?
                        .insert(field.into(), old);
                }
            }
        }
        #[derive(Deserialize)]
        struct Wire {
            state: EventState,
            contests: BTreeMap<String, StoredContest>,
            runs: Vec<Run>,
        }
        let wire: Wire = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            state: wire.state,
            contests: wire.contests,
            runs: wire.runs,
        })
    }
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

#[cfg(test)]
mod media_migration_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_templates_migrate_and_explicit_event_values_win() {
        let mut value = json!({
            "state":{"name":"e","problems":[],"teams":[],"score_freeze_time_seconds":100,"penalty_seconds":1200},
            "contests":{
                "a":{"config":{"name":"a","codes":[],"photo_url_format":"https://media/{team_login}"},"sites":{}},
                "b":{"config":{"name":"b","codes":[],"photo_url_format":"https://media/{team_login}"},"sites":{}}
            },
            "runs":[]
        });
        let event: StoredEvent = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            event.state.photo_url_format.as_deref(),
            Some("https://media/{team_login}")
        );
        let roundtrip = serde_json::to_value(event).unwrap();
        assert!(
            roundtrip["contests"]["a"]["config"]
                .get("photo_url_format")
                .is_none()
        );
        value["contests"]["b"]["config"]["photo_url_format"] =
            json!("https://different/{team_login}");
        assert!(serde_json::from_value::<StoredEvent>(value.clone()).is_err());
        value["state"]["photo_url_format"] = serde_json::Value::Null;
        let event: StoredEvent = serde_json::from_value(value).unwrap();
        assert!(event.state.photo_url_format.is_none());
    }
}
