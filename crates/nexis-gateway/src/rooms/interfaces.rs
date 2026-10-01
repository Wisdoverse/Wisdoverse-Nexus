//! HTTP interfaces for room and message use cases.

use std::time::Instant;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::metrics::{
    MESSAGES_SENT, OPERATION_ERRORS_TOTAL, OPERATION_LATENCY, OPERATION_THROUGHPUT_TOTAL,
    ROOMS_ACTIVE, ROOMS_CREATED_TOTAL,
};

use super::application::{
    CreateRoomCommand, InviteMemberCommand, RoomApplication, RoomCommandError, SendMessageCommand,
};

/// State required by the room HTTP interface.
pub trait RoomInterfaceState: Clone + Send + Sync + 'static {
    fn rooms(&self) -> &RoomApplication;
}

#[derive(Debug, Clone, Deserialize)]
struct CreateRoomRequest {
    name: String,
    #[serde(default)]
    topic: Option<String>,
    #[cfg(feature = "multi-tenant")]
    #[serde(default)]
    tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateRoomResponse {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SendMessageRequest {
    #[serde(rename = "roomId")]
    room_id: String,
    #[serde(default, rename = "sender")]
    _sender: Option<String>,
    text: String,
    #[serde(rename = "replyTo", default)]
    reply_to: Option<String>,
    #[serde(rename = "clientMessageId", default)]
    client_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct MessageResponse {
    id: String,
    #[serde(rename = "roomId")]
    room_id: String,
    sender: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RoomInfoResponse {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    messages: Vec<MessageResponse>,
    #[cfg(feature = "multi-tenant")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InviteMemberRequest {
    #[serde(rename = "memberId")]
    member_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct InviteMemberResponse {
    room_id: String,
    member_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ListRoomsResponse {
    rooms: Vec<RoomSummaryResponse>,
    total: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RoomSummaryResponse {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    member_count: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ListRoomsQuery {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

mod error_codes {
    pub const BAD_REQUEST: &str = "BAD_REQUEST";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const SERVICE_UNAVAILABLE: &str = "SERVICE_UNAVAILABLE";
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'static str>,
}

impl ErrorResponse {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: Some(error_codes::BAD_REQUEST),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: Some(error_codes::NOT_FOUND),
        }
    }

    fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            code: Some(error_codes::SERVICE_UNAVAILABLE),
        }
    }
}

/// Build room and message routes.
pub fn routes<S>() -> Router<S>
where
    S: RoomInterfaceState,
{
    Router::new()
        .route("/v1/rooms", get(list_rooms::<S>).post(create_room::<S>))
        .route(
            "/v1/rooms/{id}",
            get(get_room::<S>).delete(delete_room::<S>),
        )
        .route("/v1/rooms/{id}/invite", post(invite_member::<S>))
        .route("/v1/rooms/{id}/messages", get(list_messages::<S>))
        .route("/v1/messages", post(send_message::<S>))
        .layer(axum::middleware::from_fn(supported_mode))
}

async fn supported_mode(request: axum::extract::Request, next: axum::middleware::Next) -> Response {
    if cfg!(feature = "multi-tenant") {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "error": "M1 supports default single-tenant features only", "code": "UNSUPPORTED_MODE"
        }))).into_response();
    }
    next.run(request).await
}

fn record_operation_success(operation: &str, start: Instant) {
    OPERATION_THROUGHPUT_TOTAL
        .with_label_values(&[operation])
        .inc();
    OPERATION_LATENCY
        .with_label_values(&[operation])
        .observe(start.elapsed().as_secs_f64());
}

fn record_operation_error(operation: &str, error_type: &str, start: Instant) {
    OPERATION_ERRORS_TOTAL
        .with_label_values(&[operation, error_type])
        .inc();
    OPERATION_LATENCY
        .with_label_values(&[operation])
        .observe(start.elapsed().as_secs_f64());
}

fn room_command_error_response(
    operation: &str,
    error: RoomCommandError,
    started: Instant,
) -> Response {
    let (status, body, error_type) = match error {
        RoomCommandError::Forbidden => (
            StatusCode::FORBIDDEN,
            ErrorResponse {
                error: "room access denied".to_string(),
                code: Some("FORBIDDEN"),
            },
            "forbidden",
        ),
        RoomCommandError::Conflict => (
            StatusCode::CONFLICT,
            ErrorResponse {
                error: "retry key already used for different content".to_string(),
                code: Some("CONFLICT"),
            },
            "conflict",
        ),
        RoomCommandError::Validation(message) => (
            StatusCode::BAD_REQUEST,
            ErrorResponse::bad_request(message),
            "validation",
        ),
        RoomCommandError::RoomNotFound => (
            StatusCode::NOT_FOUND,
            ErrorResponse::not_found("room not found"),
            "room_not_found",
        ),
        RoomCommandError::ServiceUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            ErrorResponse::service_unavailable("service unavailable"),
            "unavailable",
        ),
    };
    record_operation_error(operation, error_type, started);
    (status, Json(body)).into_response()
}

#[tracing::instrument(
    name = "gateway.create_room",
    skip(state, user, payload),
    fields(room_name = %payload.name)
)]
async fn create_room<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateRoomRequest>,
) -> Response
where
    S: RoomInterfaceState,
{
    let started = Instant::now();
    let operation = "create_room";

    let command = CreateRoomCommand {
        name: payload.name,
        creator_id: Some(user.member_id.clone()),
        topic: payload.topic,
        #[cfg(feature = "multi-tenant")]
        tenant_id: payload.tenant_id,
    };

    let room = match state.rooms().create_room(command).await {
        Ok(room) => room,
        Err(error) => return room_command_error_response(operation, error, started),
    };

    let response = CreateRoomResponse {
        id: room.id,
        name: room.name,
    };

    ROOMS_CREATED_TOTAL.inc();
    ROOMS_ACTIVE.set(state.rooms().active_room_count().await as f64);
    record_operation_success(operation, started);

    (StatusCode::CREATED, Json(response)).into_response()
}

#[tracing::instrument(
    name = "gateway.send_message",
    skip(state, user, payload),
    fields(room_id = %payload.room_id)
)]
async fn send_message<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Json(payload): Json<SendMessageRequest>,
) -> Response
where
    S: RoomInterfaceState,
{
    let started = Instant::now();
    let operation = "send_message";

    if let Err(error) = state
        .rooms()
        .authorize(&payload.room_id, &user.member_id, false)
        .await
    {
        return room_command_error_response(operation, error, started);
    }
    let room_id = payload.room_id.clone();
    let command = SendMessageCommand {
        room_id: payload.room_id,
        sender: user.member_id,
        text: payload.text,
        reply_to: payload.reply_to,
    };

    let message = match state
        .rooms()
        .send_message_idempotent(command, payload.client_message_id)
        .await
    {
        Ok(message) => message,
        Err(error) => return room_command_error_response(operation, error, started),
    };

    let response = MessageResponse {
        id: message.id,
        room_id,
        sender: message.sender,
        text: message.text,
        reply_to: message.reply_to,
    };

    MESSAGES_SENT.inc();
    record_operation_success(operation, started);

    (StatusCode::CREATED, Json(response)).into_response()
}

#[tracing::instrument(
    name = "gateway.get_room",
    skip(state, user),
    fields(room_id = %id)
)]
async fn get_room<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Response
where
    S: RoomInterfaceState,
{
    let started = Instant::now();
    if let Err(error) = state.rooms().authorize(&id, &user.member_id, false).await {
        return room_command_error_response("get_room", error, started);
    }
    let details = match state.rooms().get_room(&id).await {
        Ok(details) => details,
        Err(error) => return room_command_error_response("get_room", error, started),
    };

    let room = details.room;
    let messages = details
        .messages
        .into_iter()
        .map(|message| MessageResponse {
            id: message.id,
            room_id: room.id.clone(),
            sender: message.sender,
            text: message.text,
            reply_to: message.reply_to,
        })
        .collect();
    let response = RoomInfoResponse {
        id: room.id,
        name: room.name,
        topic: room.topic,
        messages,
        #[cfg(feature = "multi-tenant")]
        tenant_id: room.tenant_id,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn list_messages<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Response
where
    S: RoomInterfaceState,
{
    let started = Instant::now();
    if let Err(error) = state.rooms().authorize(&id, &user.member_id, false).await {
        return room_command_error_response("list_messages", error, started);
    }
    let details = match state.rooms().get_room(&id).await {
        Ok(details) => details,
        Err(error) => return room_command_error_response("list_messages", error, started),
    };
    let messages: Vec<_> = details
        .messages
        .into_iter()
        .map(|message| MessageResponse {
            id: message.id,
            room_id: id.clone(),
            sender: message.sender,
            text: message.text,
            reply_to: message.reply_to,
        })
        .collect();
    (StatusCode::OK, Json(messages)).into_response()
}

#[tracing::instrument(
    name = "gateway.invite_member",
    skip(state, user, payload),
    fields(room_id = %id, member_id = %payload.member_id)
)]
async fn invite_member<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<InviteMemberRequest>,
) -> Response
where
    S: RoomInterfaceState,
{
    let started = Instant::now();
    if let Err(error) = state.rooms().authorize(&id, &user.member_id, true).await {
        return room_command_error_response("invite_member", error, started);
    }
    let operation = "invite_member";
    let result = match state
        .rooms()
        .invite_member(InviteMemberCommand {
            room_id: id,
            member_id: payload.member_id,
        })
        .await
    {
        Ok(result) => result,
        Err(error) => return room_command_error_response(operation, error, started),
    };

    let response = InviteMemberResponse {
        room_id: result.room_id,
        member_id: result.member_id,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[tracing::instrument(
    name = "gateway.list_rooms",
    skip(state, user, query),
    fields(limit = ?query.limit, offset = ?query.offset)
)]
async fn list_rooms<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Query(query): Query<ListRoomsQuery>,
) -> Response
where
    S: RoomInterfaceState,
{
    let limit = query.limit.unwrap_or(100).min(1000);
    let offset = query.offset.unwrap_or(0);
    let result = state
        .rooms()
        .list_rooms_for(&user.member_id, limit, offset)
        .await;

    let rooms = result
        .rooms
        .into_iter()
        .map(|room| RoomSummaryResponse {
            id: room.id,
            name: room.name,
            topic: room.topic,
            member_count: room.member_count,
        })
        .collect();

    let response = ListRoomsResponse {
        rooms,
        total: result.total,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[tracing::instrument(
    name = "gateway.delete_room",
    skip(state, user),
    fields(room_id = %id)
)]
async fn delete_room<S>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Response
where
    S: RoomInterfaceState,
{
    let started = Instant::now();
    if let Err(error) = state.rooms().authorize(&id, &user.member_id, true).await {
        return room_command_error_response("delete_room", error, started);
    }
    let operation = "delete_room";
    if let Err(error) = state.rooms().delete_room(&id).await {
        return room_command_error_response(operation, error, started);
    }

    (StatusCode::NO_CONTENT, ()).into_response()
}
