#![allow(dead_code)]

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::NodeStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterId(Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescriptor {
    node_id: NodeId,
    address: String,
    storage_root: PathBuf,
    capacity_bytes: Option<u64>,
    status: NodeStatus,
}