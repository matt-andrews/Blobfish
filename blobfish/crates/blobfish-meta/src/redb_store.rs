use std::io::Read;
use chrono::Utc;
use redb::{Database, Error, ReadTransaction, ReadableDatabase, ReadableTable, TableDefinition, WriteTransaction};
use uuid::Uuid;
use blobfish_core::errors::AppError;
use blobfish_core::models::bucket::Bucket;
use blobfish_core::models::object::{ChunkDescriptor, ObjectKey, ObjectVersion};
use blobfish_core::object_service::MetadataStore;
use blobfish_core::types::DbResult;

const BUCKETS: TableDefinition<&str, &[u8]> = TableDefinition::new("buckets");
const OBJECT_KEYS: TableDefinition<&str, &[u8]> = TableDefinition::new("object_keys");
const OBJECT_VERSIONS: TableDefinition<[u8; 16], &[u8]> = TableDefinition::new("object_versions");
const CHUNKS: TableDefinition<[u8; 16], &[u8]> = TableDefinition::new("chunks");
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

impl MetadataStore for RedDbStore{
    fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<DbResult> {
        let bytes = serde_json::to_vec(bucket)?;
        let txn = self.db.begin_write()?;
        let result = {
            let mut table = txn.open_table(BUCKETS)?;
            let exists = table.get(bucket.name())?.is_some();
            //we should use the exists check to make sure that created_on persists when update
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

    fn put_object(
        &self,
        key: ObjectKey,
        version: ObjectVersion,
        chunks: Vec<ChunkDescriptor>
    ) -> anyhow::Result<DbResult>{
        let txn = self.db.begin_write()?;

        let object_result = Self::put_object_key(&txn, &key)?;
        _ = Self::put_object_version(&txn, &version)?;
        _ = Self::put_object_chunks(&txn, &chunks)?;

        txn.commit()?;

        Ok(object_result)
    }

    fn get_object_data(&self, key: &str, bucket: &str) -> anyhow::Result<ObjectVersion>{
        let txn = self.db.begin_read()?;

        if(!Self::does_object_exist(&txn, &key, &bucket)?){
            return Err(anyhow::Error::from(AppError::ObjectNotFound(key.to_string())));
        }

        let key_obj = Self::get_key_read(&txn, &key, &bucket)?;
        Self::get_version(&txn, &key_obj)
    }

    fn get_object_chunks(&self, obj: ObjectVersion) -> anyhow::Result<Vec<ChunkDescriptor>> {
        let txn = self.db.begin_read()?;
        Ok(Self::get_chunks(&txn, &obj)?)
    }

    fn delete_object(&self, key: &str, bucket: &str) -> anyhow::Result<DbResult> {
        let txn = self.db.begin_write()?;
        let mut key_obj = Self::get_key_write(&txn, &key, &bucket)?;

        //Set the deleted tag so we know this is deleted
        key_obj.deleted_at = Option::from(Utc::now());

        let result = Self::put_object_key(&txn, &key_obj)?;
        txn.commit()?;

        Ok(DbResult::Deleted)
    }

}
impl RedDbStore{
    //i dont like this a whole lot - maybe should refactor?
    fn get_key(key: &str, bucket: &str) -> String{
        format!("{}/{}", bucket, key)
    }
    fn does_object_exist(txn: &ReadTransaction, key: &str, bucket: &str) -> anyhow::Result<bool> {
        let table = txn.open_table(OBJECT_KEYS)?;
        Ok(table.get(Self::get_key(&key, &bucket).as_str())?.is_some())
    }

    fn get_key_write(txn: &WriteTransaction, key: &str, bucket: &str) -> anyhow::Result<ObjectKey>{
        let table = txn.open_table(OBJECT_KEYS)?;
        let result: ObjectKey = match table.get(Self::get_key(&key, &bucket).as_str())? {
            Some(guard) => Ok(serde_json::from_slice(guard.value())?),
            None => Err(anyhow::Error::from(AppError::ObjectNotFound(key.to_string()))),
        }?;

        if result.deleted_at.is_some(){
            return Err(anyhow::Error::from(AppError::ObjectDeleted(key.to_string())));
        }

        Ok(result)
    }

    fn get_key_read(txn: &ReadTransaction, key: &str, bucket: &str) -> anyhow::Result<ObjectKey>{
        let table = txn.open_table(OBJECT_KEYS)?;
        let result: ObjectKey = match table.get(Self::get_key(&key, &bucket).as_str())? {
            Some(guard) => Ok(serde_json::from_slice(guard.value())?),
            None => Err(anyhow::Error::from(AppError::ObjectNotFound(key.to_string()))),
        }?;

        if result.deleted_at.is_some(){
            return Err(anyhow::Error::from(AppError::ObjectDeleted(key.to_string())));
        }

        Ok(result)
    }

    fn get_version(txn: &ReadTransaction, key: &ObjectKey) -> anyhow::Result<ObjectVersion>{
        let table = txn.open_table(OBJECT_VERSIONS)?;
        match table.get(key.current_version.as_bytes())? {
            Some(guard) => Ok(serde_json::from_slice(guard.value())?),
            None => Err(anyhow::Error::from(AppError::ObjectNotFound(key.key().to_string()))),
        }
    }

    fn get_chunks(txn: &ReadTransaction, obj: &ObjectVersion) -> anyhow::Result<Vec<ChunkDescriptor>>{
        let table = txn.open_table(CHUNKS)?;
        obj.chunks
            .iter()
            .map(|id| -> anyhow::Result<ChunkDescriptor> {
                let guard = table.get(id.as_bytes())?
                    .ok_or_else(|| AppError::ObjectNotFound(id.to_string()))?;
                Ok(serde_json::from_slice(guard.value())?)
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn put_object_key(txn: &WriteTransaction, key: &ObjectKey) -> anyhow::Result<DbResult>{
        let bytes = serde_json::to_vec(key)?;
        let result = {
            let real_key = Self::get_key(key.key.as_str(), key.bucket.as_str());
            let mut table = txn.open_table(OBJECT_KEYS)?;
            let exists = table.get(real_key.as_str())?.is_some();
            table.insert(real_key.as_str(), bytes.as_slice())?;
            if exists {
                DbResult::Updated
            } else {
                DbResult::Created
            }
        };

        Ok(result)
    }
    fn put_object_version(txn: &WriteTransaction, version: &ObjectVersion) -> anyhow::Result<DbResult>{
        let mut table = txn.open_table(OBJECT_VERSIONS)?;
        let exists = table.get(version.version_id.as_bytes())?.is_some();
        if exists {
            Err(anyhow::Error::from(AppError::ImmutableError(version.version_id.to_string())))
        } else {
            let bytes = serde_json::to_vec(version)?;
            table.insert(version.version_id.as_bytes(), bytes.as_slice())?;
            Ok(DbResult::Created)
        }
    }
    fn put_object_chunks(txn: &WriteTransaction, chunks: &Vec<ChunkDescriptor>) -> anyhow::Result<DbResult>{
        let mut table = txn.open_table(CHUNKS)?;

        for chunk in chunks{
            let exists = table.get(chunk.chunk_id.as_bytes())?.is_some();
            if exists {
                return Err(anyhow::Error::from(AppError::ImmutableError(chunk.chunk_id.to_string())));
            } else {
                let bytes = serde_json::to_vec(chunk)?;
                table.insert(chunk.chunk_id.as_bytes(), bytes.as_slice())?;
            }
        }

        Ok(DbResult::Created)
    }
}