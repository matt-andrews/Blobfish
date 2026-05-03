
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
}

async fn healthz() -> &'static str {
    "ok"
}