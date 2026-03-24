use axum::body::Body;
use axum::http::{Request, StatusCode};
use nexis_gateway::build_routes;
use nexis_gateway::collaboration::{
    CollaborationRateLimitKey, CollaborationRateLimitPolicy, CollaborationRateLimitScope,
};
use serde_json::Value;
use tower::ServiceExt;

fn auth_header() -> String {
    auth_header_for_subject("collab-integration-user")
}

fn auth_header_for_subject(subject: &str) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = nexis_gateway::auth::Claims {
        sub: subject.to_string(),
        exp: now + 3600,
        iat: now,
        iss: "nexis".to_string(),
        aud: "nexis".to_string(),
        member_type: "human".to_string(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("dev_only_secret_change_in_production".as_bytes()),
    )
    .expect("encode test token");

    format!("Bearer {token}")
}

fn base_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", auth_header())
        .header("x-api-version", "1")
        .body(Body::from(body.to_string()))
        .expect("request should build")
}

#[tokio::test]
async fn collaboration_document_request_response_cycle() {
    let app = build_routes();

    let create_document_response = app
        .clone()
        .oneshot(base_request(
            "POST",
            "/v1/collaboration/documents",
            serde_json::json!({
                "title": "Integration Doc",
            }),
        ))
        .await
        .expect("create document response should exist");

    assert_eq!(create_document_response.status(), StatusCode::CREATED);

    let create_document_payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(create_document_response.into_body(), usize::MAX)
            .await
            .expect("create document body should be readable"),
    )
    .expect("create document payload should parse");

    let document_id = create_document_payload["document_id"]
        .as_str()
        .expect("document_id should exist")
        .to_string();

    let sync_document_response = app
        .clone()
        .oneshot(base_request(
            "POST",
            &format!("/v1/collaboration/documents/{document_id}/sync"),
            serde_json::json!({
                "content": "Hello from integration test",
            }),
        ))
        .await
        .expect("sync document response should exist");

    assert_eq!(sync_document_response.status(), StatusCode::OK);

    let get_document_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/collaboration/documents/{document_id}/content"))
                .header("authorization", auth_header())
                .header("x-api-version", "1")
                .body(Body::empty())
                .expect("get document request should build"),
        )
        .await
        .expect("get document response should exist");

    assert_eq!(get_document_response.status(), StatusCode::OK);

    let get_document_payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(get_document_response.into_body(), usize::MAX)
            .await
            .expect("get document body should be readable"),
    )
    .expect("get document payload should parse");

    assert_eq!(
        get_document_payload["document_id"],
        Value::String(document_id),
        "document content endpoint should return same document id",
    );
}

#[tokio::test]
async fn collaboration_rejects_invalid_api_version_with_structured_error() {
    let app = build_routes();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/collaboration/meetings/rooms")
                .header("content-type", "application/json")
                .header("authorization", auth_header())
                .header("x-api-version", "2")
                .body(Body::from(
                    serde_json::json!({
                        "name": "planning-room",
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("response should exist");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable"),
    )
    .expect("payload should parse");

    assert_eq!(payload["code"], "INVALID_API_VERSION");
    assert!(payload["error"]
        .as_str()
        .expect("error should be text")
        .contains("X-API-Version"));
}

#[tokio::test]
async fn collaboration_rejects_missing_api_version_with_structured_error() {
    let app = build_routes();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/collaboration/meetings/rooms")
                .header("content-type", "application/json")
                .header("authorization", auth_header())
                .body(Body::from(
                    serde_json::json!({
                        "name": "planning-room",
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("response should exist");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable"),
    )
    .expect("payload should parse");

    assert_eq!(payload["code"], "INVALID_API_VERSION");
}

#[tokio::test]
async fn collaboration_rejects_missing_authentication() {
    let app = build_routes();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/collaboration/tasks")
                .header("content-type", "application/json")
                .header("x-api-version", "1")
                .body(Body::from(
                    serde_json::json!({
                        "title": "No auth request",
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("response should exist");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable"),
    )
    .expect("payload should parse");

    assert_eq!(payload["code"], "UNAUTHORIZED");
    assert_eq!(payload["error"], "Missing or invalid authorization token");
}

#[tokio::test]
async fn collaboration_returns_consistent_validation_error_shape() {
    let app = build_routes();

    let response = app
        .oneshot(base_request(
            "POST",
            "/v1/collaboration/calendar/events",
            serde_json::json!({
                "title": "Retro",
                "starts_at": "2026-03-04T11:00:00Z",
                "ends_at": "2026-03-04T10:00:00Z",
            }),
        ))
        .await
        .expect("response should exist");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let payload: Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body should be readable"),
    )
    .expect("payload should parse");

    assert_eq!(payload["code"], "BAD_REQUEST");
    assert_eq!(payload["error"], "starts_at must be earlier than ends_at");
}

#[tokio::test]
async fn collaboration_routes_require_versioned_path() {
    let app = build_routes();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/collaboration/meetings/rooms")
                .header("content-type", "application/json")
                .header("authorization", auth_header())
                .header("x-api-version", "1")
                .body(Body::from(
                    serde_json::json!({
                        "name": "unversioned-path",
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("response should exist");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn collaboration_rate_limit_policy_tracks_subject_scope_threshold() {
    let app = build_routes();
    let policy = CollaborationRateLimitPolicy::new(2, 60).expect("policy should be valid");
    let key = CollaborationRateLimitKey::new(
        CollaborationRateLimitScope::Tasks,
        "collab-integration-user",
    )
    .expect("rate limit key should be valid");

    let mut request_count = 0;
    for sequence in 1..=3 {
        let response = app
            .clone()
            .oneshot(base_request(
                "POST",
                "/v1/collaboration/tasks",
                serde_json::json!({
                    "title": format!("Rate Limited Task {sequence}"),
                }),
            ))
            .await
            .expect("response should exist");

        assert_eq!(response.status(), StatusCode::CREATED);
        request_count += 1;
        assert_eq!(
            policy.is_exceeded(request_count),
            sequence > 2,
            "scope {:?} subject {} should exceed on the third request",
            key.scope,
            key.subject
        );
    }

    let other_subject_key =
        CollaborationRateLimitKey::new(CollaborationRateLimitScope::Tasks, "second-user")
            .expect("second subject key should be valid");
    assert!(
        !policy.is_exceeded(1),
        "first request for {} should be allowed",
        other_subject_key.subject
    );

    let other_subject_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/collaboration/tasks")
                .header("content-type", "application/json")
                .header("authorization", auth_header_for_subject("second-user"))
                .header("x-api-version", "1")
                .body(Body::from(
                    serde_json::json!({
                        "title": "Other subject request",
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("response should exist");

    assert_eq!(other_subject_response.status(), StatusCode::CREATED);
}
