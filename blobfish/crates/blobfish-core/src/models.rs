use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    //storage: StorageConfig,
}

#[derive(Deserialize)]
pub struct NodeConfig {
    pub bind_addr: String,
    pub storage_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket{
    name: String,
    pub created_at: DateTime<Utc>,
    //versioning: VersioningMode,
}
impl Bucket {
    pub fn new(name: String) -> Self {
        Self{
            name,
            created_at: Utc::now(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}