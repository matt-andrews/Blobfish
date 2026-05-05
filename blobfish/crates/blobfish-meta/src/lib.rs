use std::path::{PathBuf};
use redb::Database;
use blobfish_core::models::Config;
use blobfish_core::object_service::Repository;
use crate::redb_store::RedDbStore;

mod redb_store;

pub fn init(config: Config) -> anyhow::Result<impl Repository>{
    let mut path = PathBuf::from(config.node.storage_root.clone());
    path.push("meta_db.redb");
    let db = Database::create(path)?;
    RedDbStore::new(db)
}
