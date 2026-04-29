//! SQLite async wrapper using spawn_blocking.

use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use tokio::task;

/// A handle to a SQLite database with WAL mode enabled.
#[derive(Clone)]
pub struct DbPool {
    path: Arc<std::path::PathBuf>,
}

impl DbPool {
    /// Open a database, enable WAL mode, run migrations, return pool handle.
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let mut conn = Connection::open(path)?;

        // Enable WAL mode
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        // Run migrations
        run_migrations(&mut conn)?;

        Ok(Self {
            path: Arc::from(path.to_path_buf()),
        })
    }

    /// Run a closure on a new connection inside `spawn_blocking`.
    pub async fn spawn_blocking<F, T>(&self, f: F) -> Result<T, rusqlite::Error>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        let path = self.path.clone();
        task::spawn_blocking(move || {
            let conn = Connection::open(&*path)?;
            f(&conn)
        })
        .await
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
    }
}

fn run_migrations(conn: &mut Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS api_keys (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            key_hash    TEXT NOT NULL UNIQUE,
            status      TEXT NOT NULL DEFAULT 'active',
            daily_limit_usd REAL NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
        CREATE INDEX IF NOT EXISTS idx_api_keys_status ON api_keys(status);

        CREATE TABLE IF NOT EXISTS request_logs (
            id                  TEXT PRIMARY KEY,
            timestamp           TEXT NOT NULL,
            api_key_id          TEXT NOT NULL REFERENCES api_keys(id),
            model               TEXT NOT NULL,
            prompt_snippet      TEXT NOT NULL,
            prompt_tokens       INTEGER NOT NULL DEFAULT 0,
            completion_tokens   INTEGER NOT NULL DEFAULT 0,
            latency_ms          REAL NOT NULL,
            status              TEXT NOT NULL,
            error_message       TEXT,
            cost_usd            REAL NOT NULL DEFAULT 0,
            provider            TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_request_logs_timestamp ON request_logs(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_request_logs_api_key ON request_logs(api_key_id, timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_request_logs_model ON request_logs(model, timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_request_logs_status ON request_logs(status, timestamp DESC);

        CREATE TABLE IF NOT EXISTS daily_usage (
            date                TEXT NOT NULL,
            api_key_id          TEXT NOT NULL REFERENCES api_keys(id),
            model               TEXT NOT NULL,
            prompt_tokens       INTEGER NOT NULL DEFAULT 0,
            completion_tokens   INTEGER NOT NULL DEFAULT 0,
            cost_usd            REAL NOT NULL DEFAULT 0,
            request_count       INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (date, api_key_id, model)
        );
        ",
    )?;

    Ok(())
}
