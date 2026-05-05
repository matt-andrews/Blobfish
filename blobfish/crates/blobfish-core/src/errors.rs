use thiserror::Error;

#[derive(Error, Debug)]
pub enum BucketError{
    #[error("Bucket name is invalid: {0}... Valid bucket name: 3–63 chars. [a-z0-9] and - only. No leading -, no trailing -, no --.")]
    InvalidBucketName(String)
}