//! Verify startup configuration in isolated processes, without mutating global test env.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn rejects_production_secret(secret: Option<&str>) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nexis-gateway"));
    command
        .env("NEXIS_ENV", "production")
        .env("NEXIS_BIND_ADDR", "127.0.0.1:0")
        .env("RUST_LOG", "error")
        .env_remove("JWT_SECRET")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(secret) = secret {
        command.env("JWT_SECRET", secret);
    }
    let mut child = command.spawn().expect("gateway binary starts");
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().expect("gateway exit status").is_none() {
        if Instant::now() >= deadline {
            child.kill().expect("terminate misconfigured gateway");
            child.wait().expect("reap misconfigured gateway");
            panic!("production gateway accepted a missing or blank JWT secret");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().expect("startup diagnostic");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("JWT_SECRET is required"));
}

#[test]
fn production_without_a_secret_fails_before_serving() {
    rejects_production_secret(None);
}

#[test]
fn production_with_a_blank_secret_fails_before_serving() {
    rejects_production_secret(Some(" \t "));
}

#[tokio::test]
async fn health_probe_works_with_production_https_redirect_enabled() {
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = reserved.local_addr().unwrap();
    drop(reserved);
    let mut child = Command::new(env!("CARGO_BIN_EXE_nexis-gateway"))
        .env("NEXIS_ENV", "production")
        .env("NEXIS_BIND_ADDR", addr.to_string())
        .env("JWT_SECRET", "synthetic-startup-probe-test-key")
        .env("NEXIS_HTTPS_REDIRECT_ENABLED", "true")
        .env("RUST_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap();
    let mut result = None;
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(response) = client.get(format!("http://{addr}/health")).send().await {
            result = Some(response);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // Always reap the child before asserting, including startup/probe failures.
    child.kill().unwrap();
    child.wait().unwrap();
    let response = result.expect("gateway health probe becomes reachable");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "OK");
}
