//! HTTP interfaces for collaboration use cases.

use axum::{
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;

use super::application::{
    AssignTaskCommand, CollaborationApplication, CollaborationApplicationError,
    CompleteTaskCommand, ConflictCheckCommand, CreateCalendarEventCommand, CreateDocumentCommand,
    CreateMeetingRoomCommand, CreateTaskCommand, GetDocumentContentCommand,
    MeetingParticipantCommand, SyncDocumentCommand,
};

#[cfg(test)]
use super::domain::{
    CollaborationRateLimitKey, CollaborationRateLimitPolicy, CollaborationRateLimitScope,
};

const API_VERSION_HEADER: &str = "x-api-version";
const SUPPORTED_API_VERSION: &str = "1";

#[derive(Debug, Clone, Deserialize)]
struct CreateMeetingRoomRequest {
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct CreateMeetingRoomResponse {
    room_id: String,
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MeetingParticipantRequest {
    user_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct MeetingParticipantResponse {
    room_id: String,
    user_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateDocumentRequest {
    title: String,
}

#[derive(Debug, Clone, Serialize)]
struct CreateDocumentResponse {
    document_id: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SyncDocumentRequest {
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct DocumentContentResponse {
    document_id: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateTaskRequest {
    title: String,
}

#[derive(Debug, Clone, Serialize)]
struct CreateTaskResponse {
    task_id: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AssignTaskRequest {
    assignee_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct TaskAssignmentResponse {
    task_id: String,
    assignee_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct CompleteTaskResponse {
    task_id: String,
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateCalendarEventRequest {
    title: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateCalendarEventResponse {
    event_id: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ConflictCheckRequest {
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
struct ConflictCheckResponse {
    has_conflicts: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CollaborationErrorResponse {
    error: String,
    code: &'static str,
}

#[derive(Debug, Clone)]
enum CollaborationError {
    BadRequest(String),
    Unauthorized,
    InvalidApiVersion,
}

impl IntoResponse for CollaborationError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(CollaborationErrorResponse {
                    error: message,
                    code: "BAD_REQUEST",
                }),
            )
                .into_response(),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(CollaborationErrorResponse {
                    error: "Missing or invalid authorization token".to_string(),
                    code: "UNAUTHORIZED",
                }),
            )
                .into_response(),
            Self::InvalidApiVersion => (
                StatusCode::BAD_REQUEST,
                Json(CollaborationErrorResponse {
                    error: "X-API-Version header is required and must be set to '1'".to_string(),
                    code: "INVALID_API_VERSION",
                }),
            )
                .into_response(),
        }
    }
}

struct CollaborationRequestContext {
    _user: AuthenticatedUser,
}

impl<S> FromRequestParts<S> for CollaborationRequestContext
where
    S: Send + Sync,
{
    type Rejection = CollaborationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state)
            .await
            .map_err(|_| CollaborationError::Unauthorized)?;

        let version_header = parts
            .headers
            .get(API_VERSION_HEADER)
            .and_then(|value| value.to_str().ok());
        if version_header != Some(SUPPORTED_API_VERSION) {
            return Err(CollaborationError::InvalidApiVersion);
        }

        Ok(Self { _user: user })
    }
}

fn bad_request_response(message: impl Into<String>) -> Response {
    CollaborationError::BadRequest(message.into()).into_response()
}

fn collaboration_application_error_response(error: CollaborationApplicationError) -> Response {
    match error {
        CollaborationApplicationError::Validation(message) => bad_request_response(message),
    }
}

/// Build collaboration routes for meeting, document, task, and calendar features.
pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/v1/collaboration/meetings/rooms",
            post(create_meeting_room),
        )
        .route(
            "/v1/collaboration/meetings/rooms/{room_id}/join",
            post(join_meeting_room),
        )
        .route(
            "/v1/collaboration/meetings/rooms/{room_id}/leave",
            post(leave_meeting_room),
        )
        .route("/v1/collaboration/documents", post(create_document))
        .route(
            "/v1/collaboration/documents/{document_id}/sync",
            post(sync_document),
        )
        .route(
            "/v1/collaboration/documents/{document_id}/content",
            get(get_document_content),
        )
        .route("/v1/collaboration/tasks", post(create_task))
        .route(
            "/v1/collaboration/tasks/{task_id}/assign",
            post(assign_task),
        )
        .route(
            "/v1/collaboration/tasks/{task_id}/complete",
            post(complete_task),
        )
        .route(
            "/v1/collaboration/calendar/events",
            post(create_calendar_event),
        )
        .route(
            "/v1/collaboration/calendar/conflicts",
            post(check_calendar_conflicts),
        )
}

async fn create_meeting_room(
    _ctx: CollaborationRequestContext,
    Json(payload): Json<CreateMeetingRoomRequest>,
) -> Response {
    let result = match CollaborationApplication::new()
        .create_meeting_room(CreateMeetingRoomCommand { name: payload.name })
    {
        Ok(result) => result,
        Err(error) => return collaboration_application_error_response(error),
    };

    let response = CreateMeetingRoomResponse {
        room_id: result.room_id,
        name: result.name,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn join_meeting_room(
    _ctx: CollaborationRequestContext,
    Path(room_id): Path<String>,
    Json(payload): Json<MeetingParticipantRequest>,
) -> Response {
    let result =
        match CollaborationApplication::new().join_meeting_room(MeetingParticipantCommand {
            room_id,
            user_id: payload.user_id,
        }) {
            Ok(result) => result,
            Err(error) => return collaboration_application_error_response(error),
        };

    let response = MeetingParticipantResponse {
        room_id: result.room_id,
        user_id: result.user_id,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn leave_meeting_room(
    _ctx: CollaborationRequestContext,
    Path(room_id): Path<String>,
    Json(payload): Json<MeetingParticipantRequest>,
) -> Response {
    let result =
        match CollaborationApplication::new().leave_meeting_room(MeetingParticipantCommand {
            room_id,
            user_id: payload.user_id,
        }) {
            Ok(result) => result,
            Err(error) => return collaboration_application_error_response(error),
        };

    let response = MeetingParticipantResponse {
        room_id: result.room_id,
        user_id: result.user_id,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn create_document(
    _ctx: CollaborationRequestContext,
    Json(payload): Json<CreateDocumentRequest>,
) -> Response {
    let result = match CollaborationApplication::new().create_document(CreateDocumentCommand {
        title: payload.title,
    }) {
        Ok(result) => result,
        Err(error) => return collaboration_application_error_response(error),
    };

    let response = CreateDocumentResponse {
        document_id: result.document_id,
        title: result.title,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn sync_document(
    _ctx: CollaborationRequestContext,
    Path(document_id): Path<String>,
    Json(payload): Json<SyncDocumentRequest>,
) -> Response {
    let result = match CollaborationApplication::new().sync_document(SyncDocumentCommand {
        document_id,
        content: payload.content,
    }) {
        Ok(result) => result,
        Err(error) => return collaboration_application_error_response(error),
    };

    let response = DocumentContentResponse {
        document_id: result.document_id,
        content: result.content,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn get_document_content(
    _ctx: CollaborationRequestContext,
    Path(document_id): Path<String>,
) -> Response {
    let result = match CollaborationApplication::new()
        .get_document_content(GetDocumentContentCommand { document_id })
    {
        Ok(result) => result,
        Err(error) => return collaboration_application_error_response(error),
    };

    let response = DocumentContentResponse {
        document_id: result.document_id,
        content: result.content,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn create_task(
    _ctx: CollaborationRequestContext,
    Json(payload): Json<CreateTaskRequest>,
) -> Response {
    let result = match CollaborationApplication::new().create_task(CreateTaskCommand {
        title: payload.title,
    }) {
        Ok(result) => result,
        Err(error) => return collaboration_application_error_response(error),
    };

    let response = CreateTaskResponse {
        task_id: result.task_id,
        title: result.title,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn assign_task(
    _ctx: CollaborationRequestContext,
    Path(task_id): Path<String>,
    Json(payload): Json<AssignTaskRequest>,
) -> Response {
    let result = match CollaborationApplication::new().assign_task(AssignTaskCommand {
        task_id,
        assignee_id: payload.assignee_id,
    }) {
        Ok(result) => result,
        Err(error) => return collaboration_application_error_response(error),
    };

    let response = TaskAssignmentResponse {
        task_id: result.task_id,
        assignee_id: result.assignee_id,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn complete_task(_ctx: CollaborationRequestContext, Path(task_id): Path<String>) -> Response {
    let result =
        match CollaborationApplication::new().complete_task(CompleteTaskCommand { task_id }) {
            Ok(result) => result,
            Err(error) => return collaboration_application_error_response(error),
        };

    let response = CompleteTaskResponse {
        task_id: result.task_id,
        status: result.status,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn create_calendar_event(
    _ctx: CollaborationRequestContext,
    Json(payload): Json<CreateCalendarEventRequest>,
) -> Response {
    let result =
        match CollaborationApplication::new().create_calendar_event(CreateCalendarEventCommand {
            title: payload.title,
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
        }) {
            Ok(result) => result,
            Err(error) => return collaboration_application_error_response(error),
        };

    let response = CreateCalendarEventResponse {
        event_id: result.event_id,
        title: result.title,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

async fn check_calendar_conflicts(
    _ctx: CollaborationRequestContext,
    Json(payload): Json<ConflictCheckRequest>,
) -> Response {
    let result =
        match CollaborationApplication::new().check_calendar_conflicts(ConflictCheckCommand {
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
        }) {
            Ok(result) => result,
            Err(error) => return collaboration_application_error_response(error),
        };
    let response = ConflictCheckResponse {
        has_conflicts: result.has_conflicts,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::auth::JwtConfig;

    #[tokio::test]
    async fn create_meeting_room_route_returns_created() {
        let token = JwtConfig::test_token("collab-user");
        let app = routes::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/collaboration/meetings/rooms")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .header("x-api-version", "1")
                    .body(Body::from(
                        json!({
                            "name": "daily-sync"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert!(payload["room_id"].as_str().unwrap().starts_with("meeting_"));
    }

    #[tokio::test]
    async fn calendar_conflict_check_route_returns_ok() {
        let token = JwtConfig::test_token("collab-user");
        let app = routes::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/collaboration/calendar/conflicts")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .header("x-api-version", "1")
                    .body(Body::from(
                        json!({
                            "starts_at": "2026-03-04T09:00:00Z",
                            "ends_at": "2026-03-04T10:00:00Z"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_meeting_room_rejects_blank_name() {
        let token = JwtConfig::test_token("collab-user");
        let app = routes::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/collaboration/meetings/rooms")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .header("x-api-version", "1")
                    .body(Body::from(
                        json!({
                            "name": "   "
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn join_meeting_room_rejects_invalid_room_id() {
        let token = JwtConfig::test_token("collab-user");
        let app = routes::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/collaboration/meetings/rooms/room$bad/join")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .header("x-api-version", "1")
                    .body(Body::from(
                        json!({
                            "user_id": "user-123"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_calendar_event_rejects_invalid_time_range() {
        let token = JwtConfig::test_token("collab-user");
        let app = routes::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/collaboration/calendar/events")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .header("x-api-version", "1")
                    .body(Body::from(
                        json!({
                            "title": "Design review",
                            "starts_at": "2026-03-04T11:00:00Z",
                            "ends_at": "2026-03-04T10:00:00Z"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rate_limit_policy_detects_exceeded_requests() {
        let policy = CollaborationRateLimitPolicy::new(100, 60).unwrap();

        assert!(!policy.is_exceeded(100));
        assert!(policy.is_exceeded(101));
    }

    #[test]
    fn rate_limit_key_rejects_invalid_subject() {
        let key =
            CollaborationRateLimitKey::new(CollaborationRateLimitScope::Documents, "user$invalid");

        assert!(key.is_err());
    }
}
