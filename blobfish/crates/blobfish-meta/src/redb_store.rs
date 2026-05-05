use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use blobfish_core::bucket::Bucket;
use blobfish_core::object_service::Repository;
use blobfish_core::types::DbResult;

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
    fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<DbResult> {
        let bytes = serde_json::to_vec(bucket)?;
        let txn = self.db.begin_write()?;
        let result = {
            let mut table = txn.open_table(BUCKETS)?;
            let exists = table.get(bucket.name())?.is_some();
            table.insert(bucket.name(), bytes.as_slice())?;
            if exists {
                DbResult::Updated
            } else {
                DbResult::Created
            }
        };
        txn.commit()?;
        Ok(result)
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

        let buckets = table
            .iter()?
            .map(|entry| -> anyhow::Result<String> {
                let (name, _) = entry?;
                Ok(name.value().to_owned())
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(buckets)
    }

    fn delete_bucket(&self, name: &str) -> anyhow::Result<DbResult> {
        let txn = self.db.begin_write()?;
        let result = {
            let mut table = txn.open_table(BUCKETS)?;
            match table.remove(name)? {
                Some(_) => DbResult::Deleted,
                None => DbResult::NotFound,
            }
        };
        txn.commit()?;
        Ok(result)
    }

    fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(BUCKETS)?;
        Ok(table.get(name)?.is_some())
    }

    fn health_check(&self) -> anyhow::Result<bool> {
        self.db.begin_read()?;
        Ok(true)
    }
}
