use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use blobfish_core::bucket::Bucket;
use blobfish_core::errors::BucketError;
use blobfish_core::object_service::ObjectService;
use blobfish_core::types::DbResult;
use crate::errors::ApiError;

pub async fn get_buckets(
    State(state): State<ObjectService>
) -> Result<(StatusCode, Json<Vec<String>>), ApiError>{
    let buckets = state.list_buckets().await?;
    Ok((StatusCode::OK, Json(buckets)))
}
pub async fn get_bucket(
    State(state): State<ObjectService>,
    Path(bucket): Path<String>
) -> Result<Response, ApiError>{
    match state.get_bucket(bucket.as_str()).await? {
        None => Ok(StatusCode::NOT_FOUND.into_response()),
        Some(b) => Ok((StatusCode::OK, Json(b)).into_response()),
    }
}
pub async fn put_bucket(
    State(state): State<ObjectService>,
    Path(bucket): Path<String>
) -> Result<StatusCode, ApiError> {
    let obj: Bucket = Bucket::new(&bucket);
    match state.put_bucket(&obj).await{
        Ok(v) => {
            match v {
                DbResult::Created => Ok(StatusCode::CREATED),
                DbResult::Updated => Ok(StatusCode::OK),
                _ => Ok(StatusCode::CONFLICT)
            }
        },
        Err(err) => {
            if let Some(e) = err.downcast_ref::<BucketError>() {
                return match e {
                    BucketError::InvalidBucketName(_) => Ok(StatusCode::BAD_REQUEST),
                }
            };
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}
pub async fn delete_bucket(
    State(state): State<ObjectService>,
    Path(bucket): Path<String>
) -> Result<StatusCode, ApiError>{
    //check for content in bucket?
    match state.delete_bucket(&bucket).await?{
        DbResult::Deleted => Ok(StatusCode::NO_CONTENT),
        DbResult::NotFound => Ok(StatusCode::NOT_FOUND),
        _ => Ok(StatusCode::CONFLICT)
    }
}