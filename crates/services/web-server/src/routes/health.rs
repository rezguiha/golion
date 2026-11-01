use axum::{Json, Router, routing::get};
use serde::Serialize;

pub fn router() -> Router {
    Router::new().route("/health", get(handler))
}

async fn handler() -> Json<Status> {
    Json(Status { status: "ok".to_owned() })
}

#[derive(Debug, Serialize)]
struct Status {
    status: String,
}
