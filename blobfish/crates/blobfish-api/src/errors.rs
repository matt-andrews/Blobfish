use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use blobfish_core::errors::BucketError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "Internal server error");
                if let Some(bucket_err) = err.downcast_ref::<BucketError>() {
                    match bucket_err{
                        BucketError::InvalidBucketName(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
                    }
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                }
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}