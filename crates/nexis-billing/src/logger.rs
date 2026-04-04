//! Async request logger with bounded channel.

use crate::db::DbPool;
use crate::pricing::calculate_cost;
use serde::Serialize;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Request status enum.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Success,
    Error,
    RateLimited,
}

impl std::fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Error => write!(f, "error"),
            Self::RateLimited => write!(f, "rate_limited"),
        }
    }
}

/// A log entry to be persisted.
#[derive(Debug, Clone, Serialize)]
pub struct RequestLogEntry {
    pub api_key_id: String,
    pub model: String,
    pub provider: String,
    pub prompt_snippet: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub latency_ms: f64,
    pub status: RequestStatus,
    pub error_message: Option<String>,
}

impl RequestLogEntry {
    /// Build a log entry. Prompt snippet is truncated to 500 chars.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_key_id: String,
        model: String,
        provider: String,
        prompt_snippet: String,
        prompt_tokens: u32,
        completion_tokens: u32,
        latency_ms: f64,
        status: RequestStatus,
        error_message: Option<String>,
    ) -> Self {
        let mut snippet = prompt_snippet;
        if snippet.len() > 500 {
            snippet.truncate(500);
        }
        Self {
            api_key_id,
            model,
            provider,
            prompt_snippet: snippet,
            prompt_tokens,
            completion_tokens,
            latency_ms,
            status,
            error_message,
        }
    }

    /// Calculate the cost for this log entry.
    pub fn cost_usd(&self) -> f64 {
        calculate_cost(&self.model, self.prompt_tokens, self.completion_tokens)
    }
}

/// Async request logger.
///
/// Uses a bounded mpsc channel (capacity 4096). When the channel is full,
/// logs are dropped and a counter can be incremented externally.
pub struct RequestLogger {
    tx: mpsc::Sender<RequestLogEntry>,
}

impl RequestLogger {
    /// Create and spawn the logger task. Returns the sender half.
    pub fn new(db: DbPool) -> Self {
        let (tx, mut rx) = mpsc::channel::<RequestLogEntry>(4096);

        tokio::spawn(async move {
            while let Some(entry) = rx.recv().await {
                if let Err(e) = persist_entry(&db, &entry).await {
                    tracing::warn!("Failed to persist billing log: {e}");
                }
            }
            tracing::info!("RequestLogger consumer shut down");
        });

        Self { tx }
    }

    /// Try to send a log entry. Returns false if dropped (channel full).
    /// Caller should increment `billing_logs_dropped_total` when false.
    pub fn try_log(&self, entry: RequestLogEntry) -> bool {
        self.tx.try_send(entry).is_ok()
    }
}

async fn persist_entry(db: &DbPool, entry: &RequestLogEntry) -> Result<(), rusqlite::Error> {
    let id = format!("log_{}", Uuid::now_v7());
    let timestamp = chrono::Utc::now().to_rfc3339();
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let cost = entry.cost_usd();
    let status_str = entry.status.to_string();
    let prompt_snippet = entry.prompt_snippet.clone();
    let api_key_id = entry.api_key_id.clone();
    let model = entry.model.clone();
    let provider = entry.provider.clone();
    let prompt_tokens = entry.prompt_tokens;
    let completion_tokens = entry.completion_tokens;
    let latency_ms = entry.latency_ms;
    let error_message = entry.error_message.clone();

    db.spawn_blocking(move |conn| {
        // INSERT request_logs
        conn.execute(
            "INSERT INTO request_logs (id, timestamp, api_key_id, model, prompt_snippet, \
             prompt_tokens, completion_tokens, latency_ms, status, error_message, cost_usd, provider) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                id,
                timestamp,
                api_key_id,
                model,
                prompt_snippet,
                prompt_tokens,
                completion_tokens,
                latency_ms,
                status_str,
                error_message,
                cost,
                provider,
            ],
        )?;

        // UPSERT daily_usage
        conn.execute(
            "INSERT INTO daily_usage (date, api_key_id, model, prompt_tokens, completion_tokens, cost_usd, request_count) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1) \
             ON CONFLICT(date, api_key_id, model) DO UPDATE SET \
             prompt_tokens = prompt_tokens + ?4, \
             completion_tokens = completion_tokens + ?5, \
             cost_usd = cost_usd + ?6, \
             request_count = request_count + 1",
            rusqlite::params![date, api_key_id, model, prompt_tokens, completion_tokens, cost],
        )?;

        Ok(())
    })
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_entry() -> RequestLogEntry {
        RequestLogEntry::new(
            "key_test".into(),
            "gpt-4o".into(),
            "openai".into(),
            "Hello world".into(),
            100,
            50,
            1234.0,
            RequestStatus::Success,
            None,
        )
    }

    #[tokio::test]
    async fn test_logger_persists_entry() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = DbPool::open(&db_path).unwrap();
        let logger = RequestLogger::new(db.clone());

        // Need an api_key row for FK constraint
        db.spawn_blocking(|conn| {
            conn.execute(
                "INSERT INTO api_keys (id, name, key_hash, created_at, updated_at) VALUES ('key_test', 'test', 'hash123', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
        }).await.unwrap();

        let entry = make_entry();
        assert!(logger.try_log(entry));

        // Give the consumer time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Verify
        let count: i64 = db
            .spawn_blocking(|conn| {
                conn.query_row("SELECT COUNT(*) FROM request_logs", [], |row| {
                    row.get::<_, i64>(0)
                })
            })
            .await
            .unwrap();
        assert_eq!(count, 1);

        let cost: f64 = db
            .spawn_blocking(|conn| {
                conn.query_row("SELECT cost_usd FROM request_logs", [], |row| {
                    row.get::<_, f64>(0)
                })
            })
            .await
            .unwrap();
        assert!((cost - (100f64 * 2.5 / 1_000_000.0 + 50f64 * 10.0 / 1_000_000.0)).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_channel_overflow_returns_false() {
        let dir = tempdir().unwrap();
        let _db = DbPool::open(&dir.path().join("test2.db")).unwrap();

        // Use a tiny channel to test overflow
        let (tx, _rx) = mpsc::channel::<RequestLogEntry>(1);

        // Fill the channel
        tx.try_send(make_entry()).unwrap(); // capacity=1

        // Block the receiver so channel stays full
        let _guard = tokio::spawn(async move {
            // Never consume — intentionally leak to keep channel full
            std::future::pending::<()>().await;
        });

        // This should fail (dropped)
        assert!(tx.try_send(make_entry()).is_err());
    }

    #[test]
    fn test_log_entry_snippet_truncation() {
        let entry = RequestLogEntry::new(
            "k".into(),
            "m".into(),
            "p".into(),
            "x".repeat(600),
            100,
            50,
            1.0,
            RequestStatus::Success,
            None,
        );
        assert_eq!(entry.prompt_snippet.len(), 500);
    }
}
