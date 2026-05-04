use redb::Database;
use crate::redb_store::RedDbStore;

pub mod redb_store;

pub fn init() -> anyhow::Result<RedDbStore>{
    let db = Database::create("my_db.redb")?;
    Ok(RedDbStore::new(db))
}
