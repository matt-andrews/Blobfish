use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use blobfish_core::errors::AppError;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Internal(err) => {
                if let Some(bucket_err) = err.downcast_ref::<AppError>() {
                    match bucket_err{
                        AppError::InvalidBucketName(msg) => {
                            tracing::warn!(error = %err, "Invalid bucket name");
                            (StatusCode::BAD_REQUEST, msg.clone())
                        },
                        AppError::ObjectNotFound(msg) => {
                            tracing::warn!(error = %err, "Object not found");
                            (StatusCode::BAD_REQUEST, msg.clone())
                        },
                        AppError::ImmutableError(msg) => {
                            tracing::warn!(error = %err, "Immutable Error");
                            (StatusCode::BAD_REQUEST, msg.clone())
                        },
                        AppError::InvalidObjectName(msg) => {
                            tracing::warn!(error = %err, "Invalid object name");
                            (StatusCode::BAD_REQUEST, msg.clone())
                        },
                        AppError::ObjectDeleted(msg) => {
                            tracing::warn!(error = %err, "Object Deleted");
                            (StatusCode::BAD_REQUEST, msg.clone())
                        },
                    }
                } else {
                    tracing::error!(error = %err, "Internal server error");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                }
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}