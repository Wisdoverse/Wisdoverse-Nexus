use axum::body::Body;
use axum::http::{Request, StatusCode};
use nexis_gateway::build_routes;
use serde_json::Value;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test_secret_for_boundary_tests";

fn auth_header() -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = nexis_gateway::auth::Claims {
        sub: "boundary-user".to_string(),
        exp: now + 3600,
        iat: now,
        iss: "nexis".to_string(),
        aud: "nexis".to_string(),
        member_type: "human".to_string(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("encode test token");

    format!("Bearer {token}")
}

async fn create_room(app: &axum::Router, auth: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/rooms")
                .header("content-type", "application/json")
                .header("authorization", auth)
                .body(Body::from(
                    serde_json::json!({
                        "name": "boundary-room",
                        "topic": "boundary-tests",
                    })
                    .to_string(),
                ))
                .expect("create room request should build"),
        )
        .await
        .expect("create room response should exist");

    assert_eq!(response.status(), StatusCode::CREATED);

    let payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("create room body"),
    )
    .expect("create room payload should parse");

    payload["id"]
        .as_str()
        .expect("room id should exist")
        .to_string()
}

#[tokio::test]
async fn rejects_oversized_messages() {
    // Set JWT_SECRET for this test
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);

    let app = build_routes();
    let auth = auth_header();
    let room_id = create_room(&app, &auth).await;

    let oversized_text = "x".repeat(32 * 1024 + 1);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("authorization", &auth)
                .body(Body::from(
                    serde_json::json!({
                        "roomId": room_id,
                        "sender": "boundary-user",
                        "text": oversized_text,
                    })
                    .to_string(),
                ))
                .expect("oversized message request should build"),
        )
        .await
        .expect("oversized message response should exist");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn supports_concurrent_message_writes() {
    // Set JWT_SECRET for this test
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);

    let app = build_routes();
    let auth = auth_header();
    let room_id = create_room(&app, &auth).await;

    let workers = (0..64usize)
        .map(|idx| {
            let app = app.clone();
            let auth = auth.clone();
            let room_id = room_id.clone();
            tokio::spawn(async move {
                app.oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/v1/messages")
                        .header("content-type", "application/json")
                        .header("authorization", auth)
                        .body(Body::from(
                            serde_json::json!({
                                "roomId": room_id,
                                "sender": format!("sender-{idx}"),
                                "text": format!("msg-{idx}"),
                            })
                            .to_string(),
                        ))
                        .expect("concurrent message request should build"),
                )
                .await
                .expect("concurrent message response should exist")
                .status()
            })
        })
        .collect::<Vec<_>>();

    for worker in workers {
        assert_eq!(
            worker.await.expect("worker should join"),
            StatusCode::CREATED
        );
    }

    let room_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/rooms/{room_id}"))
                .header("authorization", &auth)
                .body(Body::empty())
                .expect("get room request should build"),
        )
        .await
        .expect("get room response should exist");

    assert_eq!(room_response.status(), StatusCode::OK);

    let room_payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(room_response.into_body(), usize::MAX)
            .await
            .expect("get room body"),
    )
    .expect("get room payload should parse");

    assert_eq!(
        room_payload["messages"].as_array().map(|v| v.len()),
        Some(64)
    );
}

#[tokio::test]
async fn recovers_after_bad_request_and_accepts_next_message() {
    // Set JWT_SECRET for this test
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);

    let app = build_routes();
    let auth = auth_header();
    let room_id = create_room(&app, &auth).await;

    let invalid_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("authorization", &auth)
                .body(Body::from(
                    serde_json::json!({
                        "roomId": room_id,
                        "sender": "boundary-user",
                        "text": "",
                    })
                    .to_string(),
                ))
                .expect("invalid request should build"),
        )
        .await
        .expect("invalid request response should exist");

    assert_eq!(invalid_response.status(), StatusCode::BAD_REQUEST);

    let valid_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .header("authorization", &auth)
                .body(Body::from(
                    serde_json::json!({
                        "roomId": room_id,
                        "sender": "boundary-user",
                        "text": "after-error",
                    })
                    .to_string(),
                ))
                .expect("valid request should build"),
        )
        .await
        .expect("valid request response should exist");

    assert_eq!(valid_response.status(), StatusCode::CREATED);
}
