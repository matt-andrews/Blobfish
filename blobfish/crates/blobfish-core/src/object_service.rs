use std::path::{PathBuf};
use std::sync::Arc;
use anyhow::Error;
use crate::errors::BucketError;
use crate::models::Bucket;

#[derive(Clone)]
pub struct ObjectService{
    repository: Arc<dyn Repository>,
    storage_root: PathBuf
}
impl ObjectService{
    pub fn new(repository: Arc<dyn Repository>, storage_root: PathBuf)-> Self {
        Self{
            repository,
            storage_root
        }
    }
    pub async fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<()>{
        Self::is_valid_bucket_name(bucket.name())?;
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
    pub async fn delete_bucket(&self, name: &str) -> anyhow::Result<()>{
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
        let db = tokio::task::spawn_blocking(move || {
            return owned_rep.health_check();
        }).await?;

        Ok(true)
    }

    /*
    * Valid bucket name: 3–63 chars. [a-z0-9] and - only. No leading -, no trailing -, no --.
    */
    fn is_valid_bucket_name(name: &str) -> anyhow::Result<()>{
        let len = name.len();
        if(len >= 3 && len <= 63
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
            && name.bytes().all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-')))
        {
            return Ok(());
        }
        return Err(Error::from(BucketError::InvalidBucketName(name.to_string())));
    }
}
pub trait Repository: Send + Sync{
    fn put_bucket(&self, bucket: &Bucket) -> anyhow::Result<()>;
    fn get_bucket(&self, name: &str) -> anyhow::Result<Option<Bucket>>;
    fn get_all_buckets(&self) -> anyhow::Result<Vec<String>>;
    fn delete_bucket(&self, name: &str) -> anyhow::Result<()>;
    fn does_bucket_exist(&self, name: &str) -> anyhow::Result<bool>;
    fn health_check(&self) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::ObjectService;

    // --- valid ---

    #[test]
    fn valid_simple() {
        assert!(ObjectService::is_valid_bucket_name("my-bucket").is_ok());
    }

    #[test]
    fn valid_all_lowercase() {
        assert!(ObjectService::is_valid_bucket_name("abcdefghij").is_ok());
    }

    #[test]
    fn valid_all_digits() {
        assert!(ObjectService::is_valid_bucket_name("123").is_ok());
    }

    #[test]
    fn valid_min_length() {
        assert!(ObjectService::is_valid_bucket_name("abc").is_ok());
    }

    #[test]
    fn valid_max_length() {
        assert!(ObjectService::is_valid_bucket_name(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn valid_single_hyphen_middle() {
        assert!(ObjectService::is_valid_bucket_name("a-b").is_ok());
    }

    #[test]
    fn valid_multiple_hyphens_separated() {
        assert!(ObjectService::is_valid_bucket_name("a-b-c-d").is_ok());
    }

    #[test]
    fn valid_mixed_alphanumeric_and_hyphens() {
        assert!(ObjectService::is_valid_bucket_name("my-bucket-123").is_ok());
    }

    // --- length ---

    #[test]
    fn invalid_too_short_one_char() {
        assert!(ObjectService::is_valid_bucket_name("a").is_err());
    }

    #[test]
    fn invalid_too_short_two_chars() {
        assert!(ObjectService::is_valid_bucket_name("ab").is_err());
    }

    #[test]
    fn invalid_empty() {
        assert!(ObjectService::is_valid_bucket_name("").is_err());
    }

    #[test]
    fn invalid_too_long() {
        assert!(ObjectService::is_valid_bucket_name(&"a".repeat(64)).is_err());
    }

    // --- leading / trailing hyphen ---

    #[test]
    fn invalid_leading_hyphen() {
        assert!(ObjectService::is_valid_bucket_name("-bucket").is_err());
    }

    #[test]
    fn invalid_trailing_hyphen() {
        assert!(ObjectService::is_valid_bucket_name("bucket-").is_err());
    }

    #[test]
    fn invalid_leading_and_trailing_hyphen() {
        assert!(ObjectService::is_valid_bucket_name("-bucket-").is_err());
    }

    // --- double hyphen ---

    #[test]
    fn invalid_double_hyphen_middle() {
        assert!(ObjectService::is_valid_bucket_name("my--bucket").is_err());
    }

    #[test]
    fn invalid_double_hyphen_start() {
        assert!(ObjectService::is_valid_bucket_name("--bucket").is_err());
    }

    #[test]
    fn invalid_triple_hyphen() {
        assert!(ObjectService::is_valid_bucket_name("my---bucket").is_err());
    }

    // --- invalid characters ---

    #[test]
    fn invalid_uppercase() {
        assert!(ObjectService::is_valid_bucket_name("MyBucket").is_err());
    }

    #[test]
    fn invalid_all_uppercase() {
        assert!(ObjectService::is_valid_bucket_name("BUCKET").is_err());
    }

    #[test]
    fn invalid_underscore() {
        assert!(ObjectService::is_valid_bucket_name("my_bucket").is_err());
    }

    #[test]
    fn invalid_dot() {
        assert!(ObjectService::is_valid_bucket_name("my.bucket").is_err());
    }

    #[test]
    fn invalid_space() {
        assert!(ObjectService::is_valid_bucket_name("my bucket").is_err());
    }

    #[test]
    fn invalid_unicode() {
        assert!(ObjectService::is_valid_bucket_name("mü-bucket").is_err());
    }

    #[test]
    fn invalid_slash() {
        assert!(ObjectService::is_valid_bucket_name("my/bucket").is_err());
    }
}