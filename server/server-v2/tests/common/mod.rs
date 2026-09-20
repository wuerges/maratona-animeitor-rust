use service::event_store::EventStore;
use std::sync::Arc;

pub fn test_store(salt: Option<String>) -> EventStore {
    let salt = salt.unwrap_or_else(|| "test-server-salt".into());
    #[cfg(not(feature = "sqlite-tests"))]
    let database = Arc::new(database_memory::MemoryDatabase::new());
    #[cfg(feature = "sqlite-tests")]
    let database = Arc::new(TemporarySqlite::new());
    EventStore::new(database, salt)
}

#[cfg(feature = "sqlite-tests")]
struct TemporarySqlite {
    database: database_sqlite::SqliteDatabase,
    _directory: tempfile::TempDir,
}
#[cfg(feature = "sqlite-tests")]
impl TemporarySqlite {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let database =
            database_sqlite::SqliteDatabase::open(directory.path().join("test.sqlite3")).unwrap();
        Self {
            database,
            _directory: directory,
        }
    }
}
#[cfg(feature = "sqlite-tests")]
impl service::database::Database for TemporarySqlite {
    fn list(&self) -> service::database::DatabaseFuture<'_, Vec<String>> {
        self.database.list()
    }
    fn read<'a>(
        &'a self,
        name: &'a str,
    ) -> service::database::DatabaseFuture<'a, Option<service::database::StoredEvent>> {
        self.database.read(name)
    }
    fn create(
        &self,
        event: service::database::StoredEvent,
    ) -> service::database::DatabaseFuture<'_, ()> {
        self.database.create(event)
    }
    fn replace(
        &self,
        event: service::database::StoredEvent,
    ) -> service::database::DatabaseFuture<'_, ()> {
        self.database.replace(event)
    }
    fn delete<'a>(&'a self, name: &'a str) -> service::database::DatabaseFuture<'a, bool> {
        self.database.delete(name)
    }
}
