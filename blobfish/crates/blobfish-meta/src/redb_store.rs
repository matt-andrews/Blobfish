use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use blobfish_core::models::Bucket;
use blobfish_core::object_service::Repository;

const BUCKETS: TableDefinition<&str, &[u8]> = TableDefinition::new("buckets");
pub struct RedDbStore{
    db: Database,
}

impl RedDbStore {
    pub fn new(db: Database) -> anyhow::Result<Self> {
        let txn = db.begin_write()?;
        txn.open_table(BUCKETS)?;
        txn.commit()?;
        Ok(Self { db })
    }
}

impl Repository for RedDbStore{
    fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(bucket)?;
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(BUCKETS)?;
            table.insert(bucket.name(), bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    fn get_bucket(&self, name: &str) -> anyhow::Result<Option<Bucket>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(BUCKETS)?;
        match table.get(name)? {
            Some(guard) => Ok(Some(serde_json::from_slice(guard.value())?)),
            None => Ok(None),
        }
    }

    fn get_all_buckets(&self) -> anyhow::Result<Vec<String>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(BUCKETS)?;

        Ok(table
            .iter()?
            .map(|entry| entry.unwrap().0.value().to_owned())
            .collect())
    }

    fn delete_bucket(&self, name: &str) -> anyhow::Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(BUCKETS)?;
            table.remove(name)?;
        }
        txn.commit()?;
        Ok(())
    }

    fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(BUCKETS)?;
        Ok(table.get(name)?.is_some())
    }

    fn health_check(&self) -> anyhow::Result<()> {
        self.db.begin_read()?;
        Ok(())
    }
}
