//! SQLite persistence on a dedicated worker: no SQL runs on Tokio executor threads.
use rusqlite::{Connection, OptionalExtension, params};
use service::database::{Database, DatabaseError, DatabaseFuture, StoredEvent};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};

type Job = Box<dyn FnOnce(&mut Connection) + Send>;

#[derive(Clone)]
pub struct SqliteDatabase {
    jobs: Arc<mpsc::Sender<Job>>,
}

fn sql_error(error: rusqlite::Error) -> DatabaseError {
    match &error {
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy
                    | rusqlite::ErrorCode::DatabaseLocked
                    | rusqlite::ErrorCode::SystemIoFailure
                    | rusqlite::ErrorCode::DiskFull
                    | rusqlite::ErrorCode::CannotOpen
                    | rusqlite::ErrorCode::ReadOnly
            ) =>
        {
            DatabaseError::Unavailable(error.to_string())
        }
        _ => DatabaseError::Corrupt(error.to_string()),
    }
}
fn json_error(error: serde_json::Error) -> DatabaseError {
    DatabaseError::Corrupt(error.to_string())
}
fn stopped() -> DatabaseError {
    DatabaseError::Unavailable("SQLite worker stopped".into())
}

impl SqliteDatabase {
    /// Opens and migrates the database before returning. Call during startup,
    /// not in a request handler. Parent directories must already exist.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        let path = path.as_ref().to_owned();
        let (jobs, mut receiver) = mpsc::channel::<Job>(64);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("animeitor-sqlite".into())
            .spawn(move || {
                let opened = open_connection(&path);
                match opened {
                    Ok(mut connection) => {
                        if ready_tx.send(Ok(())).is_err() {
                            return;
                        }
                        while let Some(job) = receiver.blocking_recv() {
                            job(&mut connection);
                        }
                        // Closing the connection checkpoints WAL after queued work drains.
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                }
            })
            .map_err(|e| DatabaseError::Unavailable(e.to_string()))?;
        ready_rx.recv().map_err(|_| stopped())??;
        Ok(Self {
            jobs: Arc::new(jobs),
        })
    }

    async fn execute<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, DatabaseError> + Send + 'static,
    ) -> Result<T, DatabaseError> {
        let (tx, rx) = oneshot::channel();
        self.jobs
            .send(Box::new(move |connection| {
                let _ = tx.send(operation(connection));
            }))
            .await
            .map_err(|_| stopped())?;
        rx.await.map_err(|_| stopped())?
    }
}

fn open_connection(path: &Path) -> Result<Connection, DatabaseError> {
    let mut connection = Connection::open(path).map_err(sql_error)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(sql_error)?;
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(sql_error)?;
    if version > 1 {
        return Err(DatabaseError::Corrupt(format!(
            "unsupported SQLite schema version {version}"
        )));
    }
    connection
        .execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;")
        .map_err(sql_error)?;
    if version == 0 {
        let tables: i64 = connection.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r| r.get(0)).map_err(sql_error)?;
        if tables != 0 {
            return Err(DatabaseError::Corrupt(
                "unrecognized database schema".into(),
            ));
        }
        let tx = connection.transaction().map_err(sql_error)?;
        tx.execute_batch(
            "CREATE TABLE events (
            ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            configuration TEXT NOT NULL
        );
        CREATE TABLE runs (
            event TEXT NOT NULL REFERENCES events(name) ON DELETE CASCADE,
            id INTEGER NOT NULL,
            time_seconds INTEGER NOT NULL,
            payload TEXT NOT NULL,
            PRIMARY KEY(event, id)
        );
        CREATE INDEX runs_order ON runs(event, time_seconds, id);
        PRAGMA user_version=1;",
        )
        .map_err(sql_error)?;
        tx.commit().map_err(sql_error)?;
    }
    // Validate expected tables even when user_version already exists.
    connection
        .prepare("SELECT ordinal, name, configuration FROM events")
        .map_err(sql_error)?;
    connection
        .prepare("SELECT event, id, time_seconds, payload FROM runs")
        .map_err(sql_error)?;
    Ok(connection)
}

fn read_event(connection: &Connection, name: &str) -> Result<Option<StoredEvent>, DatabaseError> {
    let raw: Option<String> = connection
        .query_row(
            "SELECT configuration FROM events WHERE name=?",
            [name],
            |r| r.get(0),
        )
        .optional()
        .map_err(sql_error)?;
    let Some(raw) = raw else {
        return Ok(None);
    };
    let mut event: StoredEvent = serde_json::from_str(&raw).map_err(json_error)?;
    if event.state.name != name || !event.runs.is_empty() {
        return Err(DatabaseError::Corrupt("invalid event configuration".into()));
    }
    let mut statement = connection
        .prepare(
            "SELECT id, time_seconds, payload FROM runs WHERE event=? ORDER BY time_seconds, id",
        )
        .map_err(sql_error)?;
    let rows = statement
        .query_map([name], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(sql_error)?;
    for row in rows {
        let (id, time, raw) = row.map_err(sql_error)?;
        let run: service::event_store::Run = serde_json::from_str(&raw).map_err(json_error)?;
        if run.id != id || run.time_seconds != time {
            return Err(DatabaseError::Corrupt("run key mismatch".into()));
        }
        event.runs.push(run);
    }
    Ok(Some(event))
}

fn write_event(
    connection: &mut Connection,
    mut event: StoredEvent,
    create: bool,
) -> Result<(), DatabaseError> {
    let tx = connection.transaction().map_err(sql_error)?;
    let old = read_event(&tx, &event.state.name)?;
    match (create, old.is_some()) {
        (true, true) => return Err(DatabaseError::AlreadyExists),
        (false, false) => return Err(DatabaseError::NotFound),
        _ => (),
    }
    event.normalize()?;
    let runs = std::mem::take(&mut event.runs);
    let mut ids = std::collections::HashSet::new();
    if runs.iter().any(|r| !ids.insert(r.id)) {
        return Err(DatabaseError::Corrupt("duplicate run IDs".into()));
    }
    let configuration = serde_json::to_string(&event).map_err(json_error)?;
    let name = &event.state.name;
    if create {
        tx.execute(
            "INSERT INTO events(name, configuration) VALUES(?,?)",
            params![name, configuration],
        )
        .map_err(sql_error)?;
    } else {
        tx.execute(
            "UPDATE events SET configuration=? WHERE name=? AND configuration<>?",
            params![configuration, name, configuration],
        )
        .map_err(sql_error)?;
    }
    let old_runs: std::collections::HashMap<_, _> = old
        .as_ref()
        .map(|e| e.runs.iter().map(|r| (r.id, r)).collect())
        .unwrap_or_default();
    for run in &runs {
        if old_runs.get(&run.id).copied() == Some(run) {
            continue;
        }
        tx.execute("INSERT INTO runs(event,id,time_seconds,payload) VALUES(?,?,?,?) ON CONFLICT(event,id) DO UPDATE SET time_seconds=excluded.time_seconds,payload=excluded.payload",
            params![name, run.id, run.time_seconds, serde_json::to_string(run).map_err(json_error)?]).map_err(sql_error)?;
    }
    for id in old_runs.keys() {
        if !ids.contains(id) {
            tx.execute("DELETE FROM runs WHERE event=? AND id=?", params![name, id])
                .map_err(sql_error)?;
        }
    }
    tx.commit().map_err(sql_error)
}

impl Database for SqliteDatabase {
    fn list(&self) -> DatabaseFuture<'_, Vec<String>> {
        Box::pin(self.execute(|connection| {
            let mut statement = connection
                .prepare("SELECT name FROM events ORDER BY ordinal")
                .map_err(sql_error)?;
            statement
                .query_map([], |r| r.get(0))
                .map_err(sql_error)?
                .collect::<Result<_, _>>()
                .map_err(sql_error)
        }))
    }
    fn read<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, Option<StoredEvent>> {
        let name = name.to_owned();
        Box::pin(self.execute(move |connection| {
            let tx = connection.transaction().map_err(sql_error)?;
            let event = read_event(&tx, &name)?;
            tx.commit().map_err(sql_error)?;
            Ok(event)
        }))
    }
    fn create(&self, event: StoredEvent) -> DatabaseFuture<'_, ()> {
        Box::pin(self.execute(move |connection| write_event(connection, event, true)))
    }
    fn replace(&self, event: StoredEvent) -> DatabaseFuture<'_, ()> {
        Box::pin(self.execute(move |connection| write_event(connection, event, false)))
    }
    fn delete<'a>(&'a self, name: &'a str) -> DatabaseFuture<'a, bool> {
        let name = name.to_owned();
        Box::pin(self.execute(move |connection| {
            Ok(connection
                .execute("DELETE FROM events WHERE name=?", [name])
                .map_err(sql_error)?
                != 0)
        }))
    }
}
