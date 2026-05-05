use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::errors::BucketError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket{
    pub name: String,
    pub created_at: DateTime<Utc>,
    //versioning: VersioningMode,
}
impl Bucket {
    pub fn new(name: &str) -> Self {
        Self{
            name: name.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /*
    * Valid bucket name: 3–63 chars. [a-z0-9] and - only. No leading -, no trailing -, no --.
    */
    pub fn validate_name(&self) -> anyhow::Result<()>{
        let name = self.name();
        let len = name.len();
        if len >= 3 && len <= 63
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
            && name.bytes().all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-'))
        {
            return Ok(());
        }

        Err(Error::from(BucketError::InvalidBucketName(self.name.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::Bucket;

    // --- valid ---

    #[test]
    fn valid_simple() {
        assert!(Bucket::new("my-bucket").validate_name().is_ok());
    }

    #[test]
    fn valid_all_lowercase() {
        assert!(Bucket::new("abcdefghij").validate_name().is_ok());
    }

    #[test]
    fn valid_all_digits() {
        assert!(Bucket::new("123").validate_name().is_ok());
    }

    #[test]
    fn valid_min_length() {
        assert!(Bucket::new("abc").validate_name().is_ok());
    }

    #[test]
    fn valid_max_length() {
        assert!(Bucket::new(&"a".repeat(63)).validate_name().is_ok());
    }

    #[test]
    fn valid_single_hyphen_middle() {
        assert!(Bucket::new("a-b").validate_name().is_ok());
    }

    #[test]
    fn valid_multiple_hyphens_separated() {
        assert!(Bucket::new("a-b-c-d").validate_name().is_ok());
    }

    #[test]
    fn valid_mixed_alphanumeric_and_hyphens() {
        assert!(Bucket::new("my-bucket-123").validate_name().is_ok());
    }

    // --- length ---

    #[test]
    fn invalid_too_short_one_char() {
        assert!(Bucket::new("a").validate_name().is_err());
    }

    #[test]
    fn invalid_too_short_two_chars() {
        assert!(Bucket::new("ab").validate_name().is_err());
    }

    #[test]
    fn invalid_empty() {
        assert!(Bucket::new("").validate_name().is_err());
    }

    #[test]
    fn invalid_too_long() {
        assert!(Bucket::new(&"a".repeat(64)).validate_name().is_err());
    }

    // --- leading / trailing hyphen ---

    #[test]
    fn invalid_leading_hyphen() {
        assert!(Bucket::new("-bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_trailing_hyphen() {
        assert!(Bucket::new("bucket-").validate_name().is_err());
    }

    #[test]
    fn invalid_leading_and_trailing_hyphen() {
        assert!(Bucket::new("-bucket-").validate_name().is_err());
    }

    // --- double hyphen ---

    #[test]
    fn invalid_double_hyphen_middle() {
        assert!(Bucket::new("my--bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_double_hyphen_start() {
        assert!(Bucket::new("--bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_triple_hyphen() {
        assert!(Bucket::new("my---bucket").validate_name().is_err());
    }

    // --- invalid characters ---

    #[test]
    fn invalid_uppercase() {
        assert!(Bucket::new("MyBucket").validate_name().is_err());
    }

    #[test]
    fn invalid_all_uppercase() {
        assert!(Bucket::new("BUCKET").validate_name().is_err());
    }

    #[test]
    fn invalid_underscore() {
        assert!(Bucket::new("my_bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_dot() {
        assert!(Bucket::new("my.bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_space() {
        assert!(Bucket::new("my bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_unicode() {
        assert!(Bucket::new("mü-bucket").validate_name().is_err());
    }

    #[test]
    fn invalid_slash() {
        assert!(Bucket::new("my/bucket").validate_name().is_err());
    }
}