use serde::{Deserialize, Serialize};

pub enum DbResult{
    Created,
    Updated,
    Deleted,
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus{
    Ok,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersioningMode{
    Immutable,
}