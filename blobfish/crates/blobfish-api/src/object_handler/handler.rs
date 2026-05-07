use axum::body::{Body};
use axum::extract::{FromRequestParts, Path, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::response::IntoResponse;
use tokio_util::io::{ReaderStream, StreamReader};
use futures::TryStreamExt;
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
    let chunks = state.get_object_chunks(data.clone()).await?;
    let stream = state.storage_service.read_from_disk(chunks, &key).await?;
    let body = Body::from_stream(ReaderStream::new(stream));
    Ok((StatusCode::OK, (data_to_header(data), body)))
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
    req: Request<Body>,
) -> Result<impl IntoResponse, ApiError> {
    let (mut parts, body) = req.into_parts();

    let Path((bucket, key)) = Path::<(String, String)>::from_request_parts(&mut parts, &state)
        .await
        .map_err(|e| ApiError::Internal(anyhow::Error::from(AppError::InvalidObject(e.to_string()))))?;


    let content_type = &parts
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("application/octet-stream"));

    let stream  = body.into_data_stream().map_err(std::io::Error::other);
    let reader = StreamReader::new(stream);

    let chunks = state.storage_service.write_to_disk(reader, &key).await?;

    let status = match state.put_object(
        &key,
        &bucket,
        content_type.to_str().unwrap(),
        chunks
    ).await?{
        DbResult::Created => Ok(StatusCode::CREATED),
        DbResult::Updated => Ok(StatusCode::OK),
        _ => Err(ApiError::Internal(anyhow::Error::from(AppError::InvalidObject(key.to_string()))))
    }?;

    let data = state.get_object_data(&key, &bucket).await?;
    Ok((status, data_to_header(data), ()))
}

fn data_to_header(data: ObjectVersion) -> HeaderMap{
    let mut result = HeaderMap::new();
    let etag = data.version_id.to_string();
    result.insert("etag", format!("\"{}\"", etag).parse().unwrap());
    result.insert("x-blobfish-checksum-sha256", data.checksum_sha256.parse().unwrap());
    result.insert("content-length", data.size_bytes.to_string().parse().unwrap());
    if data.content_type.is_some() {
        result.insert("content-type", data.content_type.unwrap().to_string().parse().unwrap());
    }
    result
}