use std::sync::Arc;
use crate::bucket::Bucket;
use crate::types::DbResult;

#[derive(Clone)]
pub struct ObjectService{
    repository: Arc<dyn Repository>,
}
impl ObjectService{
    pub fn new(repository: Arc<dyn Repository>)-> Self {
        Self{
            repository,
        }
    }
    pub async fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<DbResult>{
        bucket.validate_name()?;
        let owned_rep = Arc::clone(&self.repository);
        let owned_bucket = bucket.to_owned();
        tokio::task::spawn_blocking(move || {
            return owned_rep.put_bucket(&owned_bucket);
        }).await?
    }
    pub async fn get_bucket(&self, name: &str) -> anyhow::Result<Option<Bucket>>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            return owned_rep.get_bucket(&owned_name);
        }).await?
    }
    pub async fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            return owned_rep.does_bucket_exist(&owned_name);
        }).await?
    }
    pub async fn delete_bucket(&self, name: &str) -> anyhow::Result<DbResult>{
        let owned_rep = Arc::clone(&self.repository);
        let owned_name = name.to_owned();
        tokio::task::spawn_blocking(move || {
            return owned_rep.delete_bucket(&owned_name);
        }).await?
    }
    pub async fn list_buckets(&self) -> anyhow::Result<Vec<String>>{
        let owned_rep = Arc::clone(&self.repository);
        tokio::task::spawn_blocking(move || {
            return owned_rep.get_all_buckets();
        }).await?
    }
    pub async fn health_check(&self) -> anyhow::Result<bool>{
        let owned_rep = Arc::clone(&self.repository);
        tokio::task::spawn_blocking(move || {
            return owned_rep.health_check();
        }).await?
    }


}
pub trait Repository: Send + Sync{
    fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<DbResult>;
    fn get_bucket(&self, name: &str) -> anyhow::Result<Option<Bucket>>;
    fn get_all_buckets(&self) -> anyhow::Result<Vec<String>>;
    fn delete_bucket(&self, name: &str) -> anyhow::Result<DbResult>;
    fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool>;
    fn health_check(&self) -> anyhow::Result<bool>;
}