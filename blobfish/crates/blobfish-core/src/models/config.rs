use std::path::PathBuf;
use serde::{Deserialize};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub node: NodeConfig,
    pub metadata: MetadataConfig,
}

#[derive(Deserialize, Clone)]
pub struct NodeConfig {
    pub bind_addr: String,
    pub storage_root: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct MetadataConfig {
    pub engine: String,
    pub path: PathBuf
}