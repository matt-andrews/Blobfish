use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    BadBucketRequest(#[from] BucketError),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum BucketError{
    #[error("Bad request: {0}")]
    InvalidBucketName(String)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::BadBucketRequest(e) => {
                tracing::debug!(msg = %e, "Bad request");
                (StatusCode::BAD_REQUEST, self.to_string())
            },
            ApiError::BadRequest(s) => {
                tracing::debug!(msg = s, "Bad request");
                (StatusCode::BAD_REQUEST, self.to_string())
            },
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}