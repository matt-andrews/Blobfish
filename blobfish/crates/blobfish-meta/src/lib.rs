use redb::Database;
use blobfish_core::models::config::Config;
use blobfish_core::object_service::MetadataStore;
use crate::redb_store::RedDbStore;

mod redb_store;

pub fn init(config: Config) -> anyhow::Result<impl MetadataStore>{
    let db = Database::create(config.metadata.path)?;
    RedDbStore::new(db)
}
