use axum::body::Body;
use axum::http::{Request, StatusCode};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use nexis_gateway::auth::Claims;
use nexis_gateway::build_routes;
use tower::ServiceExt;

fn authorization_header() -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: "bench-user".to_string(),
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
    .expect("benchmark token should encode");

    format!("Bearer {token}")
}

fn create_room_request(auth: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/rooms")
        .header("content-type", "application/json")
        .header("authorization", auth)
        .body(Body::from(
            serde_json::json!({
                "name": "bench-throughput-room",
                "topic": "bench",
            })
            .to_string(),
        ))
        .expect("create-room request should build")
}

fn send_message_request(room_id: &str, sender: &str, text: &str, auth: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .header("authorization", auth)
        .body(Body::from(
            serde_json::json!({
                "roomId": room_id,
                "sender": sender,
                "text": text,
            })
            .to_string(),
        ))
        .expect("send-message request should build")
}

fn benchmark_message_throughput(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should build");

    let auth = authorization_header();
    let mut group = c.benchmark_group("message/throughput");

    for batch in [1usize, 10, 100] {
        group.throughput(Throughput::Elements(batch as u64));
        group.bench_function(BenchmarkId::new("send_batch", batch), |b| {
            b.iter_batched(
                || {
                    runtime.block_on(async {
                        let app = build_routes();
                        let create_response = app
                            .clone()
                            .oneshot(create_room_request(&auth))
                            .await
                            .expect("create room should respond");

                        assert_eq!(create_response.status(), StatusCode::CREATED);

                        let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
                            .await
                            .expect("create room body should read");
                        let payload: serde_json::Value =
                            serde_json::from_slice(&body).expect("create room payload should parse");
                        (app, payload["id"].as_str().expect("room id should exist").to_string())
                    })
                },
                |(app, room_id)| {
                    runtime.block_on(async {
                        for idx in 0..batch {
                            let response = app
                                .clone()
                                .oneshot(send_message_request(
                                    &room_id,
                                    "bench",
                                    &format!("message-{idx}"),
                                    &auth,
                                ))
                                .await
                                .expect("send message should respond");
                            assert_eq!(response.status(), StatusCode::CREATED);
                        }
                    });
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(message_benches, benchmark_message_throughput);
criterion_main!(message_benches);
