use std::path::PathBuf;
use serde::Deserialize;

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