use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use blobfish_core::errors::AppError;
use blobfish_core::models::object::ObjectVersion;
use blobfish_core::object_service::ObjectService;
use blobfish_core::types::DbResult;
use crate::errors::ApiError;

pub async fn get_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<impl IntoResponse, ApiError> {
    let data = state.get_object_data(&key, &bucket).await?;
    Ok((StatusCode::OK, (data_to_header(data), ())))
}

pub async fn head_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<impl IntoResponse, ApiError> {
    let data = state.get_object_data(&key, &bucket).await?;
    Ok((StatusCode::OK, (data_to_header(data), ())))
}

pub async fn delete_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<StatusCode, ApiError>{
    match state.delete_object(&key, &bucket).await? {
        DbResult::Deleted => Ok(StatusCode::NO_CONTENT),
        _ => Err(ApiError::Internal(anyhow::Error::from(AppError::ObjectNotFound(key.to_string()))))
    }
}

pub async fn put_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<impl IntoResponse, ApiError> {
    let status = match state.put_object(&key, &bucket).await?{
        DbResult::Created => Ok(StatusCode::CREATED),
        DbResult::Updated => Ok(StatusCode::OK),
        _ => Err(ApiError::Internal(anyhow::Error::from(AppError::ObjectNotFound(key.to_string()))))
    }?;
    let data = state.get_object_data(&key, &bucket).await?;
    Ok((status, data_to_header(data), ()))
}

fn data_to_header(data: ObjectVersion) -> HeaderMap{
    let mut result = HeaderMap::new();
    result.insert("ETag", "\"{data.version_id.to_string()}\"".parse().unwrap());
    result.insert("X-Blobfish-Checksum-Sha256", data.checksum_sha256.parse().unwrap());
    result.insert("Content-Length", data.size_bytes.to_string().parse().unwrap());
    if data.content_type.is_some() {
        result.insert("Content-Type", data.content_type.unwrap().to_string().parse().unwrap());
    }
    result
}