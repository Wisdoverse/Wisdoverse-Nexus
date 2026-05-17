//! Privacy infrastructure adapters.

#[cfg(feature = "persistence-sqlx")]
use tracing::{info, instrument};
#[cfg(feature = "persistence-sqlx")]
use uuid::Uuid;

/// Permanently delete data that has passed the retention period.
///
/// When enabled, this should be called by a scheduled task / cron job.
#[cfg(feature = "persistence-sqlx")]
#[instrument(name = "gdpr.purge_expired", skip(pool))]
pub async fn purge_expired_deletions(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
    info!("Running GDPR data purge for expired retention periods");

    let mut tx = pool.begin().await?;

    let expired: Vec<Uuid> = sqlx::query_scalar(
        r#"
        SELECT member_id
        FROM deletion_queue
        WHERE scheduled_at <= NOW()
          AND status = 'pending'
        FOR UPDATE SKIP LOCKED
        LIMIT 100
        "#,
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut count = 0u64;
    for mid in &expired {
        sqlx::query("DELETE FROM messages WHERE member_id = $1")
            .bind(mid)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM members WHERE id = $1")
            .bind(mid)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            "UPDATE deletion_queue SET status = 'completed', completed_at = NOW() WHERE member_id = $1",
        )
        .bind(mid)
        .execute(&mut *tx)
        .await?;

        count += 1;
        info!("Permanently deleted member: {}", mid);
    }

    tx.commit().await?;
    info!(
        "GDPR purge completed: {} members permanently deleted",
        count
    );
    Ok(count)
}
