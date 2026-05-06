use std::sync::Arc;
use crate::models::bucket::Bucket;
use crate::models::object::{ChunkDescriptor, ObjectKey, ObjectVersion};
use crate::types::DbResult;

#[derive(Clone)]
pub struct ObjectService{
    repository: Arc<dyn MetadataStore>,
}
impl ObjectService{
    pub fn new(repository: Arc<dyn MetadataStore>) -> Self {
        Self{
            repository,
        }
    }
    pub async fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<DbResult>{
        bucket.validate_name()?;
        let owned_rep = Arc::clone(&self.repository);
        let owned_bucket = bucket.to_owned();
        tokio::task::spawn_blocking(move || {
            owned_rep.put_bucket(&owned_bucket)
        }).await?
    }
    pub async fn get_bucket(&self, name: &str) -> anyhow::Result<Option<Bucket>>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            owned_rep.get_bucket(&owned_name)
        }).await?
    }
    pub async fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            return owned_rep.does_bucket_exist(&owned_name)
        }).await?
    }
    pub async fn delete_bucket(&self, name: &str) -> anyhow::Result<DbResult>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            owned_rep.delete_bucket(&owned_name)
        }).await?
    }
    pub async fn list_buckets(&self) -> anyhow::Result<Vec<String>>{
        let owned_rep = Arc::clone(&self.repository);
        tokio::task::spawn_blocking(move || {
            owned_rep.get_all_buckets()
        }).await?
    }
    pub async fn health_check(&self) -> anyhow::Result<bool>{
        let owned_rep = Arc::clone(&self.repository);
        tokio::task::spawn_blocking(move || {
            owned_rep.health_check()
        }).await?
    }
    pub async fn put_object(&self, key: &str, bucket: &str) -> anyhow::Result<DbResult>{
        let owned_rep = Arc::clone(&self.repository);
        let key_obj = ObjectKey::new(key, bucket);
        let version = ObjectVersion::new(key_obj.key_id);
        let chunks: Vec<ChunkDescriptor> = vec![];
        tokio::task::spawn_blocking(move || {
            owned_rep.put_object(key_obj, version, chunks)
        }).await?
    }
    pub async fn get_object_data(&self, key: &str, bucket: &str) -> anyhow::Result<ObjectVersion>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_key = key.to_string();
        let owned_bucket = bucket.to_string();
        tokio::task::spawn_blocking(move || {
            owned_rep.get_object_data(&owned_key, &owned_bucket)
        }).await?
    }
    pub async fn delete_object(&self, key: &str, bucket: &str) -> anyhow::Result<DbResult>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_key = key.to_string();
        let owned_bucket = bucket.to_string();
        tokio::task::spawn_blocking(move || {
            owned_rep.delete_object(&owned_key, &owned_bucket)
        }).await?
    }

}
pub trait MetadataStore: Send + Sync{
    fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<DbResult>;
    fn get_bucket(&self, name: &str) -> anyhow::Result<Option<Bucket>>;
    fn get_all_buckets(&self) -> anyhow::Result<Vec<String>>;
    fn delete_bucket(&self, name: &str) -> anyhow::Result<DbResult>;
    fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool>;
    fn health_check(&self) -> anyhow::Result<bool>;
    fn put_object(
        &self,
        key: ObjectKey,
        version: ObjectVersion,
        chunks: Vec<ChunkDescriptor>
    ) -> anyhow::Result<DbResult>;
    fn get_object_data(&self, key: &str, bucket: &str) -> anyhow::Result<ObjectVersion>;
    fn delete_object(&self, key: &str, bucket: &str) -> anyhow::Result<DbResult>;
}