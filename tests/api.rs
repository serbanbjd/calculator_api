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

fn post(body: &str) -> Request<Body> {
    Request::post("/v1/calculate")
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

async fn calculate(body: &str) -> (StatusCode, Value) {
    send(post(body)).await
}

fn assert_error(res: (StatusCode, Value), status: StatusCode, code: &str, field: Option<&str>) {
    let (got_status, body) = res;
    assert_eq!(got_status, status, "{body}");
    assert_eq!(body["error"]["code"], code, "{body}");
    assert!(body["error"]["message"].is_string(), "{body}");
    match field {
        Some(f) => assert_eq!(body["error"]["field"], f, "{body}"),
        None => assert!(body["error"].get("field").is_none(), "{body}"),
    }
}

// FR-1..FR-8 through HTTP: one success per operation, result as a string (ADR-0002).
#[tokio::test]
async fn every_operation_succeeds() {
    let cases = [
        (r#"{"operation":"add","operands":[2,3]}"#, "5"),
        (r#"{"operation":"subtract","operands":[5,3]}"#, "2"),
        (r#"{"operation":"multiply","operands":[4,2.5]}"#, "10"),
        (
            r#"{"operation":"divide","operands":[1,"3"]}"#,
            "0.3333333333333333333333333333",
        ),
        (r#"{"operation":"power","operands":[2,10]}"#, "1024"),
        (
            r#"{"operation":"sqrt","operands":[2]}"#,
            "1.4142135623730950488",
        ),
        (r#"{"operation":"modulo","operands":[-10,3]}"#, "-1"),
        (r#"{"operation":"percentage","operands":[20,50]}"#, "10"),
    ];
    for (body, expected) in cases {
        let (status, res) = calculate(body).await;
        assert_eq!(status, StatusCode::OK, "{body}: {res}");
        assert_eq!(res, serde_json::json!({ "result": expected }), "{body}");
    }
}

// NFR-3: JSON numbers are read from their text, never through f64.
#[tokio::test]
async fn json_numbers_are_exact_decimals() {
    let (_, res) = calculate(r#"{"operation":"add","operands":[0.1,0.2]}"#).await;
    assert_eq!(res["result"], "0.3");
    let (_, res) = calculate(r#"{"operation":"add","operands":[1e3,"2.5e-3"]}"#).await;
    assert_eq!(res["result"], "1000.0025");
}

// FR-9: shape problems are INVALID_INPUT naming the field at fault.
#[tokio::test]
async fn invalid_requests_name_the_field() {
    let bad = StatusCode::BAD_REQUEST;
    let cases = [
        (r#"{"operation":"evaluate","operands":[1,2]}"#, "operation"),
        (r#"{"operation":"add","operands":[1]}"#, "operands"),
        (r#"{"operation":"sqrt","operands":[1,2]}"#, "operands"),
        (r#"{"operation":"add","operands":[1,true]}"#, "operands[1]"),
        (r#"{"operation":"add","operands":[null,1]}"#, "operands[0]"),
        (r#"{"operation":"add","operands":["abc",1]}"#, "operands[0]"),
        (r#"{"operation":"add","operands":[1,1e400]}"#, "operands[1]"),
    ];
    for (body, field) in cases {
        assert_error(calculate(body).await, bad, "INVALID_INPUT", Some(field));
    }
}

// FR-9: unknown and missing fields are rejected.
#[tokio::test]
async fn unknown_or_missing_fields_are_invalid_input() {
    for body in [
        r#"{"operation":"add","operands":[1,2],"precision":5}"#,
        r#"{"operands":[1,2]}"#,
        r#"{"operation":"add"}"#,
    ] {
        let (status, res) = calculate(body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(res["error"]["code"], "INVALID_INPUT", "{body}");
    }
}

// ADR-0002: maths errors are 422 with their PRD code.
#[tokio::test]
async fn maths_errors_are_422() {
    let unprocessable = StatusCode::UNPROCESSABLE_ENTITY;
    for (body, code) in [
        (
            r#"{"operation":"divide","operands":[1,0]}"#,
            "DIVISION_BY_ZERO",
        ),
        (
            r#"{"operation":"modulo","operands":[1,0]}"#,
            "DIVISION_BY_ZERO",
        ),
        (
            r#"{"operation":"sqrt","operands":[-1]}"#,
            "UNDEFINED_RESULT",
        ),
        (
            r#"{"operation":"power","operands":[-8,0.5]}"#,
            "UNDEFINED_RESULT",
        ),
        (
            r#"{"operation":"multiply","operands":[1e20,1e10]}"#,
            "RESULT_OUT_OF_RANGE",
        ),
    ] {
        assert_error(calculate(body).await, unprocessable, code, None);
    }
}

// FR-9, NFR-4: nothing reaches the client as plain text.
#[tokio::test]
async fn malformed_requests_get_json_errors() {
    let bad = StatusCode::BAD_REQUEST;
    assert_error(calculate("{not json").await, bad, "INVALID_INPUT", None);
    assert_error(calculate("").await, bad, "INVALID_INPUT", None);
    assert_error(calculate("[1,2]").await, bad, "INVALID_INPUT", None);

    let no_content_type = Request::post("/v1/calculate")
        .body(Body::from(r#"{"operation":"add","operands":[1,2]}"#))
        .unwrap();
    assert_error(send(no_content_type).await, bad, "INVALID_INPUT", None);
}

// NFR-5: bodies over 16 KB are 413 with INVALID_INPUT.
#[tokio::test]
async fn oversized_body_is_413() {
    let body = format!(
        r#"{{"operation":"add","operands":[1,2],"pad":"{}"}}"#,
        "x".repeat(17 * 1024)
    );
    assert_error(
        calculate(&body).await,
        StatusCode::PAYLOAD_TOO_LARGE,
        "INVALID_INPUT",
        None,
    );
}

// ADR-0002: unknown paths and wrong methods also answer in JSON.
#[tokio::test]
async fn unknown_path_and_wrong_method_are_json() {
    assert_error(
        send(get("/nope")).await,
        StatusCode::NOT_FOUND,
        "INVALID_INPUT",
        None,
    );
    assert_error(
        send(get("/v1/calculate")).await,
        StatusCode::METHOD_NOT_ALLOWED,
        "INVALID_INPUT",
        None,
    );
}

// FR-12: every executable example in the OpenAPI contract holds.
#[tokio::test]
async fn openapi_examples_hold() {
    let spec = include_str!("../docs/api/openapi.yaml");
    assert!(spec.starts_with("openapi: 3.1"));
    let examples: Vec<Value> = spec
        .split_once("\nx-examples:\n")
        .expect("x-examples section")
        .1
        .lines()
        .filter_map(|l| l.trim().strip_prefix("- "))
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(examples.len() >= 10, "found {}", examples.len());
    for ex in examples {
        let (status, body) = calculate(&ex["request"].to_string()).await;
        assert_eq!(status.as_u16(), ex["status"], "{ex}");
        assert_eq!(body, ex["response"], "{ex}");
    }
}
