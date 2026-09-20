# Database backends

One server process owns each database. Backend selection is in `server.toml`;
CLI tools parse the same file but continue to manage the server over HTTPS.

## Memory (default)

Omit `[database]`, or use:

```toml
[database]
type = "memory"
```

Each database instance is isolated and all event data disappears on restart.

## SQLite

Create the parent directory (`mkdir -p var`) and add:

```toml
[database]
type = "sqlite"
path = "var/animeitor.sqlite3"
```

Relative paths resolve against the directory containing `server.toml`, not the
shell's working directory. The server creates a missing database file and applies
schema migrations automatically. Invalid paths, corrupt schemas, or newer schema
versions fail clearly; storage never silently falls back to memory.

SQLite preserves event configuration, ordered teams/problems, contests, sites,
salts, runs (including corrections), event creation order, and the last explicitly
set timer value. The clock does not advance itself while the server is stopped.
Keep the same `revelation_salt` to preserve revelation URLs. Remote-control
commands are transient and are not replayed after restart.

Use a local filesystem and run only one Animeitor server against a database.
Direct external writes while the server runs are unsupported. SQLite's normal
transactions protect individual operations; they do not coordinate independent
Animeitor servers or their WebSocket channels.

### Docker volume

Uncomment the SQLite block in a local copy of `server.docker.toml.example`, with
`path = "/data/animeitor.sqlite3"`. Mount that configuration and a named volume:

```yaml
services:
  animeitor:
    volumes:
      - ./server.docker.local.toml:/workspace/server.toml:ro
      - animeitor-data:/data
volumes:
  animeitor-data:
```

Use this as an override of the existing Compose stack; it replaces the server
configuration mount at the same target. Build the project image containing the
SQLite backend before starting it. Preserve the named volume across container
recreation; `docker compose down -v` deletes it.

### Backups and recovery

Stop the server cleanly, then copy the database and any remaining `-wal`, `-shm`,
or `-journal` sidecars together. Store `server.toml` securely alongside your backup
if you need to recover the same revelation salt. Do not copy only the main file
while the server is writing: recent commits may still be in the WAL. Restore only
while the server is stopped. No automatic import of a running memory backend is
provided; populate SQLite through the administrative API or feeder.

### Durability and errors

SQLite uses WAL mode, `synchronous=FULL`, foreign keys, and a five-second busy
timeout. Every accepted mutation is committed before the service publishes it.
HTTP client cancellation does not interrupt a mutation already in progress.
Configuration and runs are stored separately; timer updates do not rewrite runs.

The APIs return `503 storage_unavailable` for unavailable/busy storage and
`500 storage_error` for corrupt or unsupported data. Responses do not expose file
paths or underlying SQL. A write error can have an ambiguous commit outcome:
read the resource before retrying. The service reloads authoritative data and
closes affected old streams when recovering from a failed commit; clients must
reconnect. Database errors and latency are exposed as `database_errors_total`
and `database_operation_seconds` metrics.

## Extending and testing

`service::database::Database` is an asynchronous, object-safe persistence trait.
`database-memory` and `database-sqlite` implement it in separate crates. Domain
validation, API projections, filtering, and live channels stay in `service`.
The server selects and injects the backend. Implementations must preserve event
creation order and atomic operations and distinguish missing data from failures.

```sh
make test-databases
```

This runs shared database contracts, service and CLI tests, then the server
endpoint suites against memory and temporary SQLite files. The `sqlite-tests`
feature affects test fixtures only; it does not select the production backend.
Tests include restart recovery, rollback, write cancellation, ambiguous commits,
error envelopes, and checks that timer updates leave run rows untouched.
