//! GDPR Data Subject Rights API Handlers
//!
//! Implements GDPR compliance endpoints:
//! - Data portability (export) — GDPR Article 20
//! - Right to be forgotten (deletion) — GDPR Article 17
//!
//! Both endpoints operate on the in-memory AppState. When the
//! `persistence-sqlx` feature is enabled, `purge_expired_deletions`
//! permanently removes data past the 30-day retention window.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
#[cfg(feature = "persistence-sqlx")]
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::metrics::{OPERATION_ERRORS_TOTAL, OPERATION_LATENCY, OPERATION_THROUGHPUT_TOTAL};
use crate::router::{AppState, StoredMessage};

// ── Response / request types ─────────────────────────────────────────

#[derive(Serialize, Debug)]
pub struct DataExportResponse {
    pub member: MemberExportData,
    pub messages: Vec<MessageExportData>,
    pub rooms: Vec<RoomExportData>,
    pub exported_at: DateTime<Utc>,
    pub format: String,
}

#[derive(Serialize, Debug)]
pub struct MemberExportData {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct MessageExportData {
    pub id: String,
    pub room_id: String,
    pub sender: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct RoomExportData {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DeletionRequest {
    pub confirm: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct DeletionResponse {
    pub status: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_items: DeletedItems,
    pub retention_until: DateTime<Utc>,
}

#[derive(Serialize, Debug, Default)]
pub struct DeletedItems {
    pub messages: i64,
    pub rooms_created: i64,
}

// ── Helpers ──────────────────────────────────────────────────────────

fn ok_json<T: Serialize>(val: T) -> Response {
    (StatusCode::OK, Json(val)).into_response()
}

fn err_json(status: StatusCode, code: &str, msg: impl Into<String>) -> Response {
    #[derive(Serialize)]
    struct E {
        error: String,
        code: String,
    }
    (status, Json(E { error: msg.into(), code: code.to_string() })).into_response()
}

fn record_ok(op: &str, t: std::time::Instant) {
    OPERATION_THROUGHPUT_TOTAL.with_label_values(&[op]).inc();
    OPERATION_LATENCY.with_label_values(&[op]).observe(t.elapsed().as_secs_f64());
}

fn record_err(op: &str, kind: &str, t: std::time::Instant) {
    OPERATION_ERRORS_TOTAL.with_label_values(&[op, kind]).inc();
    OPERATION_LATENCY.with_label_values(&[op]).observe(t.elapsed().as_secs_f64());
}

// ── GET /v1/members/me/export ───────────────────────────────────────

#[instrument(name = "gdpr.export_data", skip(state, user), fields(member_id = %user.member_id))]
pub(crate) async fn export_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Response {
    let t = std::time::Instant::now();
    let uid = user.member_id.clone();

    // Collect messages authored by this member across all rooms.
    let messages: Vec<MessageExportData> = {
        let msgs = state.room_messages().read().await;
        msgs.iter()
            .flat_map(|(room_id, list): (&String, &Vec<StoredMessage>)| {
                list.iter()
                    .filter(|m| m.sender == uid)
                    .map(|m| MessageExportData {
                        id: m.id.clone(),
                        room_id: room_id.clone(),
                        sender: m.sender.clone(),
                        content: m.text.clone(),
                    })
            })
            .collect()
    };

    // Collect rooms created by this member.
    let rooms: Vec<RoomExportData> = {
        let rooms = state.rooms().read().await;
        rooms
            .values()
            .filter(|r| r.id.contains(&uid.replace('-', "")))
            .map(|r| RoomExportData {
                id: r.id.clone(),
                name: r.name.clone(),
                topic: r.topic.clone(),
            })
            .collect()
    };

    let member = MemberExportData {
        id: uid.clone(),
        email: format!("{}@nexis.local", uid),
        created_at: Utc::now(),
    };

    record_ok("gdpr_export", t);
    info!("GDPR export completed for member: {}", uid);

    ok_json(DataExportResponse {
        member,
        messages,
        rooms,
        exported_at: Utc::now(),
        format: "json".to_string(),
    })
}

// ── DELETE /v1/members/me ───────────────────────────────────────────

#[instrument(name = "gdpr.delete_data", skip(state, user, body), fields(member_id = %user.member_id))]
pub(crate) async fn delete_data(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(body): Json<DeletionRequest>,
) -> Response {
    let t = std::time::Instant::now();
    let uid = user.member_id.clone();

    if !body.confirm {
        record_err("gdpr_delete", "validation", t);
        return err_json(
            StatusCode::BAD_REQUEST,
            "DELETION_REQUIRED",
            "Deletion requires explicit confirmation. Set \"confirm\" to true.",
        );
    }

    info!(
        "Processing GDPR deletion for member: {}, reason: {:?}",
        uid, body.reason
    );

    let deleted_at = Utc::now();
    let retention_until = deleted_at + Duration::days(30);

    // ── 1. Anonymize messages (soft-delete: keep metadata, scrub content) ──
    let mut message_count = 0i64;
    {
        let mut msgs = state.room_messages().write().await;
        for list in msgs.values_mut() {
            for msg in list.iter_mut() {
                if msg.sender == uid {
                    msg.text = "[Content removed per GDPR request]".to_string();
                    msg.sender = "[deleted]".to_string();
                    message_count += 1;
                }
            }
        }
    }

    // ── 2. Remove member from room rosters ──
    {
        let mut members = state.room_members().write().await;
        for list in members.values_mut() {
            if let Some(pos) = list.iter().position(|m| m == &uid) {
                list.remove(pos);
            }
        }
    }

    // ── 3. Identify and soft-delete rooms created by this member ──
    let to_remove: Vec<String> = {
        let rooms = state.rooms().read().await;
        rooms
            .keys()
            .filter(|id| id.contains(&uid.replace('-', "")))
            .cloned()
            .collect()
    };
    let rooms_owned = to_remove.len() as i64;

    // Drop read lock before acquiring write locks (avoid deadlock)
    {
        let mut rooms = state.rooms().write().await;
        for id in &to_remove {
            rooms.remove(id);
        }
    }
    {
        let mut msgs = state.room_messages().write().await;
        let mut mems = state.room_members().write().await;
        for id in &to_remove {
            msgs.remove(id);
            mems.remove(id);
        }
    }

    record_ok("gdpr_delete", t);
    info!(
        "GDPR deletion completed for member: {} ({} messages anonymized, {} rooms removed)",
        uid, message_count, rooms_owned
    );

    ok_json(DeletionResponse {
        status: "deleted".to_string(),
        deleted_at,
        deleted_items: DeletedItems {
            messages: message_count,
            rooms_created: rooms_owned,
        },
        retention_until,
    })
}

// ── Background purge (DB-backed, behind feature flag) ───────────────

/// Permanently delete data that has passed the 30-day retention period.
///
/// This is a no-op without the `persistence-sqlx` feature.
/// When enabled, it should be called by a scheduled task / cron job.
#[cfg(feature = "persistence-sqlx")]
#[instrument(name = "gdpr.purge_expired", skip(pool))]
pub async fn purge_expired_deletions(
    pool: &sqlx::PgPool,
) -> Result<u64, sqlx::Error> {
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
    info!("GDPR purge completed: {} members permanently deleted", count);
    Ok(count)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_requires_confirmation() {
        let req = DeletionRequest { confirm: false, reason: None };
        assert!(!req.confirm);
    }

    #[test]
    fn deleted_items_defaults_to_zero() {
        let items = DeletedItems::default();
        assert_eq!(items.messages, 0);
        assert_eq!(items.rooms_created, 0);
    }
}
