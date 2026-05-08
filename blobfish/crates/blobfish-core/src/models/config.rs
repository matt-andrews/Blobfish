use std::path::PathBuf;
use serde::{Deserialize};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub node: NodeConfig,
    pub metadata: MetadataConfig,
    pub storage: StorageConfig
}

#[derive(Deserialize, Clone)]
pub struct NodeConfig {
    pub bind_addr: String,
}

#[derive(Deserialize, Clone)]
pub struct MetadataConfig {
    pub engine: String,
    pub path: PathBuf
}

#[derive(Deserialize, Clone)]
pub struct StorageConfig{
    pub chunk_dir: PathBuf,
    pub chunk_size_bytes: usize,
    pub perm_dir: PathBuf,
}