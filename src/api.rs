//! HTTP layer: routes, request/response types, error mapping (ADR-0002).

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, rejection::JsonRejection},
    http::StatusCode,
    http::header,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::calc::{self, CalcError, Operation};

/// The full application router. Used by `main` and by the API tests.
pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/calculate", post(calculate))
        .route("/openapi.yaml", get(openapi))
        .route("/docs", get(|| async { Html(include_str!("docs.html")) }))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(DefaultBodyLimit::max(16 * 1024)) // NFR-5
        // NFR-7: one span per request (method, path) and a response line with
        // status and latency. Bodies, and so operands, are never logged.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}

async fn not_found() -> ApiError {
    ApiError {
        status: StatusCode::NOT_FOUND,
        ..ApiError::invalid("no such endpoint".into(), None)
    }
}

async fn method_not_allowed() -> ApiError {
    ApiError {
        status: StatusCode::METHOD_NOT_ALLOWED,
        ..ApiError::invalid("method not allowed on this endpoint".into(), None)
    }
}

// FR-14: the same contract file the API tests check (ADR-0006).
async fn openapi() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/yaml")],
        include_str!("../docs/api/openapi.yaml"),
    )
}

// FR-11
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CalculateRequest {
    operation: String,
    operands: Vec<Value>,
}

async fn calculate(
    body: Result<Json<CalculateRequest>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let Json(req) = body.map_err(|r| ApiError {
        // 413 for the body limit (NFR-5); every other rejection is a bad request.
        status: if r.status() == StatusCode::PAYLOAD_TOO_LARGE {
            StatusCode::PAYLOAD_TOO_LARGE
        } else {
            StatusCode::BAD_REQUEST
        },
        ..ApiError::invalid(r.body_text(), None)
    })?;
    let op = Operation::from_name(&req.operation).ok_or_else(|| {
        ApiError::invalid(
            format!("unknown operation '{}'", req.operation),
            Some("operation".into()),
        )
    })?;
    if req.operands.len() != op.arity() {
        return Err(ApiError::invalid(
            format!(
                "{} takes {} operand(s), got {}",
                req.operation,
                op.arity(),
                req.operands.len()
            ),
            Some("operands".into()),
        ));
    }
    let operands = req
        .operands
        .iter()
        .enumerate()
        .map(|(i, v)| {
            operand(v).map_err(|e| ApiError::invalid(e.to_string(), Some(format!("operands[{i}]"))))
        })
        .collect::<Result<Vec<_>, _>>()?;
    tracing::info!(operation = req.operation, "calculate");
    let result = calc::evaluate(op, &operands)?;
    // ADR-0002: always a string, so no client parses it into a lossy double.
    Ok(Json(json!({ "result": result.to_string() })))
}

/// A JSON number (raw text, thanks to `arbitrary_precision`) or a numeric string.
fn operand(v: &Value) -> Result<calc::Decimal, CalcError> {
    match v {
        Value::Number(n) => calc::parse_operand(&n.to_string()),
        Value::String(s) => calc::parse_operand(s),
        _ => Err(CalcError::InvalidOperand(
            "must be a number or a numeric string",
        )),
    }
}

/// The error body from ADR-0002: `{"error":{"code","message","field"?}}`.
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    field: Option<String>,
}

impl ApiError {
    fn invalid(message: String, field: Option<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "INVALID_INPUT",
            message,
            field,
        }
    }
}

impl From<CalcError> for ApiError {
    fn from(e: CalcError) -> Self {
        let status = match e {
            CalcError::InvalidOperand(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        Self {
            status,
            code: e.code(),
            message: e.to_string(),
            field: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut error = json!({ "code": self.code, "message": self.message });
        if let Some(field) = self.field {
            error["field"] = json!(field);
        }
        (self.status, Json(json!({ "error": error }))).into_response()
    }
}
