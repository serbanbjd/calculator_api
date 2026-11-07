//! API tests: run the real router in-process through `app()` (ADR-0005).
#![allow(clippy::unwrap_used)] // test helpers may panic: a panic is a failed test

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

async fn send(req: Request<Body>) -> (StatusCode, Value) {
    let res = calculator_api::app().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

// FR-11
#[tokio::test]
async fn health_returns_ok() {
    let (status, body) = send(get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({ "status": "ok" }));
}
