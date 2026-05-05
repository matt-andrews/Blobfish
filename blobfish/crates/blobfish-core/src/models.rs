use std::path::PathBuf;
use serde::{Deserialize};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub node: NodeConfig,
    //storage: StorageConfig,
}

#[derive(Deserialize, Clone)]
pub struct NodeConfig {
    pub bind_addr: String,
    pub storage_root: PathBuf,
}