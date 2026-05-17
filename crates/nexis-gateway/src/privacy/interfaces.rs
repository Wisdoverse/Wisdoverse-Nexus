//! HTTP interfaces for privacy and GDPR data subject rights.
//!
//! Implements:
//! - Data portability (export) — GDPR Article 20
//! - Right to be forgotten (deletion) — GDPR Article 17

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::auth::AuthenticatedUser;
use crate::metrics::{OPERATION_ERRORS_TOTAL, OPERATION_LATENCY, OPERATION_THROUGHPUT_TOTAL};

use super::application::{DeleteMemberDataCommand, PrivacyApplication, PrivacyApplicationError};

/// State required by the privacy HTTP interface.
pub trait PrivacyInterfaceState: Clone + Send + Sync + 'static {
    fn privacy(&self) -> &PrivacyApplication;
}

#[derive(Serialize, Debug)]
struct DataExportResponse {
    member: MemberExportData,
    messages: Vec<MessageExportData>,
    rooms: Vec<RoomExportData>,
    exported_at: DateTime<Utc>,
    format: String,
}

#[derive(Serialize, Debug)]
struct MemberExportData {
    id: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
struct MessageExportData {
    id: String,
    room_id: String,
    sender: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct RoomExportData {
    id: String,
    name: String,
    topic: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DeletionRequest {
    confirm: bool,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Serialize, Debug)]
struct DeletionResponse {
    status: String,
    deleted_at: DateTime<Utc>,
    deleted_items: DeletedItemsResponse,
    retention_until: DateTime<Utc>,
}

#[derive(Serialize, Debug, Default)]
struct DeletedItemsResponse {
    messages: i64,
    rooms_created: i64,
}

/// Build privacy routes.
pub fn routes<S>() -> Router<S>
where
    S: PrivacyInterfaceState,
{
    Router::new()
        .route("/v1/members/me/export", get(export_data::<S>))
        .route("/v1/members/me", delete(delete_data::<S>))
}

fn ok_json<T: Serialize>(val: T) -> Response {
    (StatusCode::OK, Json(val)).into_response()
}

fn err_json(status: StatusCode, code: &str, msg: impl Into<String>) -> Response {
    #[derive(Serialize)]
    struct ErrorPayload {
        error: String,
        code: String,
    }

    (
        status,
        Json(ErrorPayload {
            error: msg.into(),
            code: code.to_string(),
        }),
    )
        .into_response()
}

fn record_ok(op: &str, started: std::time::Instant) {
    OPERATION_THROUGHPUT_TOTAL.with_label_values(&[op]).inc();
    OPERATION_LATENCY
        .with_label_values(&[op])
        .observe(started.elapsed().as_secs_f64());
}

fn record_err(op: &str, kind: &str, started: std::time::Instant) {
    OPERATION_ERRORS_TOTAL.with_label_values(&[op, kind]).inc();
    OPERATION_LATENCY
        .with_label_values(&[op])
        .observe(started.elapsed().as_secs_f64());
}

#[instrument(name = "gdpr.export_data", skip(state, user), fields(member_id = %user.member_id))]
async fn export_data<S>(State(state): State<S>, user: AuthenticatedUser) -> Response
where
    S: PrivacyInterfaceState,
{
    let started = std::time::Instant::now();
    let member_id = user.member_id.clone();

    let export = state.privacy().export_member_data(&member_id).await;
    let messages = export
        .messages
        .into_iter()
        .map(|message| MessageExportData {
            id: message.id,
            room_id: message.room_id,
            sender: message.sender,
            content: message.content,
        })
        .collect();
    let rooms = export
        .rooms
        .into_iter()
        .map(|room| RoomExportData {
            id: room.id,
            name: room.name,
            topic: room.topic,
        })
        .collect();

    let member = MemberExportData {
        id: export.member.id,
        email: export.member.email,
        created_at: export.member.created_at,
    };

    record_ok("gdpr_export", started);
    info!("GDPR export completed for member: {}", member_id);

    ok_json(DataExportResponse {
        member,
        messages,
        rooms,
        exported_at: export.exported_at,
        format: export.format,
    })
}

#[instrument(name = "gdpr.delete_data", skip(state, user, body), fields(member_id = %user.member_id))]
async fn delete_data<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Json(body): Json<DeletionRequest>,
) -> Response
where
    S: PrivacyInterfaceState,
{
    let started = std::time::Instant::now();
    let member_id = user.member_id.clone();

    let deletion = match state
        .privacy()
        .delete_member_data(DeleteMemberDataCommand {
            member_id: member_id.clone(),
            confirm: body.confirm,
            reason: body.reason,
        })
        .await
    {
        Ok(deletion) => deletion,
        Err(PrivacyApplicationError::ConfirmationRequired) => {
            record_err("gdpr_delete", "validation", started);
            return err_json(
                StatusCode::BAD_REQUEST,
                "DELETION_REQUIRED",
                "Deletion requires explicit confirmation. Set \"confirm\" to true.",
            );
        }
        Err(PrivacyApplicationError::ServiceUnavailable) => {
            record_err("gdpr_delete", "unavailable", started);
            return err_json(
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                "service unavailable",
            );
        }
        Err(PrivacyApplicationError::Unexpected) => {
            record_err("gdpr_delete", "unexpected", started);
            return err_json(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred. Please try again later.",
            );
        }
    };

    record_ok("gdpr_delete", started);
    info!(
        "GDPR deletion completed for member: {} ({} messages anonymized, {} rooms removed)",
        member_id, deletion.deleted_items.messages, deletion.deleted_items.rooms_created
    );

    ok_json(DeletionResponse {
        status: "deleted".to_string(),
        deleted_at: deletion.deleted_at,
        deleted_items: DeletedItemsResponse {
            messages: deletion.deleted_items.messages,
            rooms_created: deletion.deleted_items.rooms_created,
        },
        retention_until: deletion.retention_until,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::auth::JwtConfig;
    use crate::rooms::RoomApplication;

    #[derive(Clone)]
    struct TestState {
        privacy: PrivacyApplication,
    }

    impl Default for TestState {
        fn default() -> Self {
            Self {
                privacy: PrivacyApplication::new(RoomApplication::default()),
            }
        }
    }

    impl PrivacyInterfaceState for TestState {
        fn privacy(&self) -> &PrivacyApplication {
            &self.privacy
        }
    }

    #[test]
    fn deletion_requires_confirmation() {
        let request = DeletionRequest {
            confirm: false,
            reason: None,
        };

        assert!(!request.confirm);
    }

    #[test]
    fn deleted_items_defaults_to_zero() {
        let items = DeletedItemsResponse::default();

        assert_eq!(items.messages, 0);
        assert_eq!(items.rooms_created, 0);
    }

    #[tokio::test]
    async fn export_data_route_returns_member_payload() {
        let token = JwtConfig::test_token("privacy-user");
        let app = routes::<TestState>().with_state(TestState::default());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/members/me/export")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["member"]["id"], "privacy-user");
        assert_eq!(payload["format"], "json");
    }

    #[tokio::test]
    async fn delete_data_requires_confirmation_route() {
        let token = JwtConfig::test_token("privacy-user");
        let app = routes::<TestState>().with_state(TestState::default());

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/v1/members/me")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "confirm": false
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["code"], "DELETION_REQUIRED");
    }
}
