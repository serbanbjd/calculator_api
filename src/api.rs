//! HTTP layer: routes, request/response types, error mapping (ADR-0002).

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

/// The full application router. Used by `main` and by the API tests.
pub fn app() -> Router {
    Router::new().route("/health", get(health))
}

// FR-11
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
