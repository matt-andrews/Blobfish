
use axum::{Router, routing::get};
use axum::extract::State;
use axum::http::StatusCode;
use tracing::warn;
use blobfish_core::object_service::ObjectService;
use crate::{buckets_handler, object_handler};
use crate::errors::ApiError;

pub fn router(object_service: ObjectService) -> Router {
    let mut router : Router<ObjectService> = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));

    router = buckets_handler::router(router);
    router = object_handler::router(router);

    router
        .with_state(object_service)
}

async fn healthz() -> &'static str {
    "ok"
}
async fn readyz(
    State(state): State<ObjectService>
) -> Result<StatusCode, ApiError> {
    match state.health_check().await{
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => {
            warn!(%err, "readyz failed");
            Ok(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}