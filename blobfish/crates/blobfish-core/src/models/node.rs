use std::path::PathBuf;
use uuid::Uuid;
use crate::types::NodeStatus;

pub struct ClusterId(Uuid);
pub struct NodeId(Uuid);

pub struct NodeDescriptor {
    node_id: NodeId,
    address: String,
    storage_root: PathBuf,
    capacity_bytes: Option<u64>,
    status: NodeStatus,
}