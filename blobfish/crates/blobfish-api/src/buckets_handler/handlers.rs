use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use blobfish_core::models::Bucket;
use blobfish_core::object_service::ObjectService;
use blobfish_core::errors::ApiError;

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
) -> Result<StatusCode, ApiError>{
    let existing= state.get_bucket(&bucket).await?;

    match existing.is_none(){
        true => {
            let obj: Bucket = Bucket::new(bucket);
            state.put_bucket(&obj).await?;
            Ok(StatusCode::CREATED)
        },
        false => Ok(StatusCode::OK),
    }

}
pub async fn delete_bucket(
    State(state): State<ObjectService>,
    Path(bucket): Path<String>
) -> Result<StatusCode, ApiError>{
    //check for content?
    state.delete_bucket(&bucket).await?;
    Ok(StatusCode::NO_CONTENT)
}