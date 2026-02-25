use axum::body::Body;
use axum::http::{Request, StatusCode};
use nexis_gateway::build_routes;
use serde_json::Value;
use tower::ServiceExt;

fn auth_header() -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = nexis_gateway::auth::Claims {
        sub: "integration-user".to_string(),
        exp: now + 3600,
        iat: now,
        iss: "nexis".to_string(),
        aud: "nexis".to_string(),
        member_type: "human".to_string(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("default_secret".as_bytes()),
    )
    .expect("encode test token");
    format!("Bearer {token}")
}

#[tokio::test]
async fn api_create_room_and_send_message_roundtrip() {
    let app = build_routes();
    let auth = auth_header();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/rooms")
                .header("content-type", "application/json")
                .header("authorization", auth.clone())
                .body(Body::from(
                    serde_json::json!({
                        "name": "integration-room",
                        "topic": "api",
                    })
                    .to_string(),
                ))
                .expect("create room request should build"),
        )
        .await
        .expect("create room response");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let room_payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("create room body"),
    )
    .expect("create room payload should parse");
    let room_id = room_payload["id"]
        .as_str()
        .expect("room id should exist")
        .to_string();

    let send_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("authorization", auth.clone())
                .body(Body::from(
                    serde_json::json!({
                        "roomId": room_id,
                        "sender": "integration-user",
                        "text": "integration message",
                    })
                    .to_string(),
                ))
                .expect("send message request should build"),
        )
        .await
        .expect("send message response");

    assert_eq!(send_response.status(), StatusCode::CREATED);

    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/rooms/{room_id}"))
                .header("authorization", auth)
                .body(Body::empty())
                .expect("get room request should build"),
        )
        .await
        .expect("get room response");

    assert_eq!(get_response.status(), StatusCode::OK);

    let room_snapshot: Value = serde_json::from_slice(
        &axum::body::to_bytes(get_response.into_body(), usize::MAX)
            .await
            .expect("get room body"),
    )
    .expect("room snapshot should parse");
    assert_eq!(room_snapshot["messages"].as_array().map(|v| v.len()), Some(1));
    assert_eq!(room_snapshot["messages"][0]["text"], "integration message");
}
