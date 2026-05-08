use axum::body::{Body};
use axum::extract::{FromRequestParts, Path, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::response::IntoResponse;
use tokio_util::io::{ReaderStream, StreamReader};
use futures::TryStreamExt;
use blobfish_core::errors::AppError;
use blobfish_core::models::object::{ChunkDescriptor, ObjectVersion};
use blobfish_core::object_service::ObjectService;
use blobfish_core::types::DbResult;
use crate::errors::ApiError;

pub async fn get_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<impl IntoResponse, ApiError> {
    let data = state.get_object_data(&key, &bucket).await?;
    let chunks = state.get_object_chunks(data.clone()).await?;
    let stream = state.storage_service.read_from_disk(chunks.clone(), &key).await?;
    let body = Body::from_stream(ReaderStream::new(stream));
    Ok((StatusCode::OK, (data_to_header(data, Option::from(chunks)), body)))
}

pub async fn head_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<impl IntoResponse, ApiError> {
    let data = state.get_object_data(&key, &bucket).await?;
    Ok((StatusCode::OK, (data_to_header(data, None), ())))
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
        .map_err(|e| ApiError::Internal(anyhow::Error::from(AppError::InvalidObject(e.to_string(), Option::from(e.to_string())))))?;

    let headers = &parts.headers;

    let content_type = headers
        .get("content-type")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("application/octet-stream"));

    let sha256_input = headers
        .get("x-blobfish-sha256")
        .cloned();

    let stream  = body.into_data_stream().map_err(std::io::Error::other);
    let reader = StreamReader::new(stream);

    let chunks = state.storage_service.write_to_disk(reader, &key).await?;

    if let Some(sha256_input) = sha256_input {
        let full_sha: String = chunks.iter().map(|i| i.chunk_id.clone()).collect();
        if full_sha != sha256_input {
            return Err(ApiError::Internal(anyhow::Error::from(AppError::IntegrityValidationFailed(format!("gen: {} || input: {}", full_sha, sha256_input.to_str().unwrap_or_default())))))
        }
    }

    let status = match state.put_object(
        &key,
        &bucket,
        content_type.to_str().unwrap_or_default(),
        chunks.clone()
    ).await?{
        DbResult::Created => Ok(StatusCode::CREATED),
        DbResult::Updated => Ok(StatusCode::OK),
        _ => Err(ApiError::Internal(anyhow::Error::from(AppError::InvalidObject(key.to_string(), Option::from("not db created or updated".to_string())))))
    }?;

    let data = state.get_object_data(&key, &bucket).await?;
    Ok((status, data_to_header(data, Option::from(chunks)), ()))
}

fn data_to_header(data: ObjectVersion, chunks: Option<Vec<ChunkDescriptor>>) -> HeaderMap{
    let mut result = HeaderMap::new();
    //etag is intentionally a {sha256_of_sha256_hashes}-{count} for future proofing chunk compat
    result.insert("etag", format!("\"{}\"", data.checksum_sha256).parse().unwrap());
    result.insert("content-length", data.size_bytes.to_string().parse().unwrap());
    if let Some(chunks) = chunks{
        let full_sha: String = chunks.iter().map(|i| i.chunk_id.clone()).collect();
        //this is the full concatenated sha256 of all the chunks
        result.insert("x-blobfish-sha256", full_sha.parse().unwrap());
    }
    if data.content_type.is_some() {
        result.insert("content-type", data.content_type.unwrap().to_string().parse().unwrap());
    }
    result
}