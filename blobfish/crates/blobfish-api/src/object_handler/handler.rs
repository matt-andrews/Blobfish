use axum::extract::{Path, State};
use blobfish_core::object_service::ObjectService;
use crate::errors::ApiError;

pub async fn get_object(
    State(state): State<ObjectService>,
    Path((bucket, key)): Path<(String, String)>
) -> Result<(), ApiError>{

    Ok(())
}

pub async fn head_object(
    State(state): State<ObjectService>
) -> Result<(), ApiError>{

    Ok(())
}

pub async fn delete_object(
    State(state): State<ObjectService>
) -> Result<(), ApiError>{

    Ok(())
}

pub async fn put_object(
    State(state): State<ObjectService>
) -> Result<(), ApiError>{

    Ok(())
}