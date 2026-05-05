use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::node::NodeId;

pub struct ObjectKey{
    pub key: String,
    pub current_version: Uuid,
}

pub struct ObjectVersion {
    bucket: String,
    key: ObjectKey,
    version_id: Uuid,
    size_bytes: u64,
    content_type: Option<String>,
    etag: String,
    checksum_sha256: String,
    manifest_id: Uuid,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}
pub struct ChunkDescriptor {
    chunk_id: Uuid,
    ordinal: u32,
    offset: u64,
    size_bytes: u64,
    checksum_sha256: String,
    node_id: NodeId,
    local_path: PathBuf,
}

pub struct ObjectManifest {
    manifest_id: Uuid,
    version_id: Uuid,
    chunks: Vec<ChunkDescriptor>,
    total_size_bytes: u64,
    checksum_sha256: String,
}