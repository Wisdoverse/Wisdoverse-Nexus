//! Versioned HTTP run contracts and a stateless read-only MCP JSON-RPC endpoint.

use super::{
    application::{AgentApplication, RunActor},
    domain::{InvokeAgent, RunError},
};
use crate::auth::AuthenticatedUser;
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

pub trait AgentInterfaceState: Clone + Send + Sync + 'static {
    fn agents(&self) -> &AgentApplication;
}

pub fn routes<S: AgentInterfaceState>() -> Router<S> {
    Router::new()
        .route("/v1/rooms/{room_id}/agent-runs", post(invoke::<S>))
        .route("/v1/rooms/{room_id}/agents", get(capabilities::<S>))
        .route("/v1/rooms/{room_id}/agent-runs/{run_id}", get(read::<S>))
        .route(
            "/v1/rooms/{room_id}/agent-runs/{run_id}/events",
            get(events::<S>),
        )
        .route(
            "/v1/rooms/{room_id}/agent-runs/{run_id}/cancel",
            post(cancel::<S>),
        )
        .route("/v1/rooms/{room_id}/mcp", post(mcp::<S>))
        .layer(DefaultBodyLimit::max(65_536))
}

fn actor(user: &AuthenticatedUser) -> Result<RunActor, RunError> {
    if cfg!(feature = "multi-tenant") {
        return Err(RunError::UnsupportedMode);
    }
    Ok(RunActor {
        member_id: user.member_id.clone(),
        expires_at: i64::try_from(user.claims.exp).unwrap_or(i64::MAX),
    })
}

async fn capabilities<S: AgentInterfaceState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(room): Path<String>,
) -> Response {
    let actor = match actor(&user) {
        Ok(actor) => actor,
        Err(e) => return error(e),
    };
    if let Err(e) = state.agents().authorize_scope(&room, &actor).await {
        return error(e);
    }
    Json(json!({"contractVersion":"1.0","agentId":"nexis:ai:room-assistant","agentName":"Room Assistant","enabled":state.agents().enabled(),"contextMessageLimit":20,"maxOutputTokens":4096,"maxToolCalls":1,"provider":"openai-compatible","disclosure":"Your prompt and selected room messages are sent to the configured model. Responses are read-only."})).into_response()
}

async fn invoke<S: AgentInterfaceState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(room): Path<String>,
    Json(command): Json<InvokeAgent>,
) -> Response {
    if user.member_type != "human" {
        return error(RunError::Forbidden);
    }
    let actor = match actor(&user) {
        Ok(actor) => actor,
        Err(e) => return error(e),
    };
    match state.agents().invoke(&room, actor, command).await {
        Ok(run) => (StatusCode::ACCEPTED, Json(run)).into_response(),
        Err(e) => error(e),
    }
}

async fn read<S: AgentInterfaceState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path((room, run)): Path<(String, String)>,
) -> Response {
    let actor = match actor(&user) {
        Ok(actor) => actor,
        Err(e) => return error(e),
    };
    match state.agents().get(&room, &run, &actor).await {
        Ok(run) => Json(run).into_response(),
        Err(e) => error(e),
    }
}

#[derive(Default, Deserialize)]
struct EventCursor {
    #[serde(default)]
    after: u64,
}

async fn events<S: AgentInterfaceState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path((room, run)): Path<(String, String)>,
    Query(cursor): Query<EventCursor>,
) -> Response {
    let actor = match actor(&user) {
        Ok(actor) => actor,
        Err(e) => return error(e),
    };
    match state.agents().get(&room, &run, &actor).await {
        Ok(run) => Json(json!({"events":run.events.into_iter().filter(|event| event.sequence > cursor.after).collect::<Vec<_>>(), "status":run.status})).into_response(),
        Err(e) => error(e),
    }
}

async fn cancel<S: AgentInterfaceState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path((room, run)): Path<(String, String)>,
) -> Response {
    let actor = match actor(&user) {
        Ok(actor) => actor,
        Err(e) => return error(e),
    };
    match state.agents().cancel(&room, &run, &actor).await {
        Ok(run) => Json(run).into_response(),
        Err(e) => error(e),
    }
}

fn error(e: RunError) -> Response {
    let status = match e {
        RunError::Forbidden => StatusCode::FORBIDDEN,
        RunError::NotFound => StatusCode::NOT_FOUND,
        RunError::Disabled | RunError::Capacity | RunError::UnsupportedMode => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        RunError::Provider => StatusCode::BAD_GATEWAY,
        RunError::Conflict => StatusCode::CONFLICT,
        RunError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, Json(json!({"error":e.code(),"code":e.code()}))).into_response()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

async fn mcp<S: AgentInterfaceState>(
    State(state): State<S>,
    user: AuthenticatedUser,
    Path(room): Path<String>,
    Json(request): Json<RpcRequest>,
) -> Response {
    let actor = match actor(&user) {
        Ok(actor) => actor,
        Err(e) => return error(e),
    };
    if let Err(e) = state.agents().authorize_scope(&room, &actor).await {
        return error(e);
    }
    let id = request.id;
    if request.jsonrpc != "2.0" || !(id.is_null() || id.is_string() || id.is_number()) {
        return Json(json!({"jsonrpc":"2.0","id":Value::Null,"error":{"code":-32600,"message":"Invalid request"}})).into_response();
    }
    let result = match request.method.as_str() {
        "initialize" => Ok(
            json!({"protocolVersion":"2025-03-26","capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"nexus-room-tools","version":"1.0"}}),
        ),
        "notifications/initialized" if id.is_null() => return StatusCode::ACCEPTED.into_response(),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(
            json!({"tools":[{"name":"room_history","description":"Read up to 20 messages from this authorized room; never performs writes.","annotations":{"readOnlyHint":true,"destructiveHint":false,"openWorldHint":false},"inputSchema":{"type":"object","properties":{},"additionalProperties":false}}]}),
        ),
        "tools/call" => {
            if request.params["name"] != "room_history"
                || !request
                    .params
                    .get("arguments")
                    .is_none_or(|args| args.as_object().is_some_and(serde_json::Map::is_empty))
            {
                Err((-32602, "Tool name or arguments denied"))
            } else {
                match state
                    .agents()
                    .read_tool(&room, &actor, "room_history", &[])
                    .await
                {
                    Ok(sources) => Ok(
                        json!({"content":[{"type":"text","text":json!({"sources":sources}).to_string()}],"isError":false}),
                    ),
                    Err(_) => Err((-32001, "Room access denied")),
                }
            }
        }
        _ => Err((-32601, "Method not supported")),
    };
    match result {
        Ok(result) => Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response(),
        Err((code, message)) => {
            Json(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}))
                .into_response()
        }
    }
}
