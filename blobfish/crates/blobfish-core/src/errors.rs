use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bucket name is invalid: {0}... Valid bucket name: 3–63 chars. [a-z0-9] and - only. No leading -, no trailing -, no --.")]
    InvalidBucketName(String),

    #[error("Object is immutable. Duplicate object: {0}")]
    ImmutableError(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    /// returns 404
    #[error("Object has been deleted: {0}")]
    ObjectDeleted(String),

    #[error("Object name is invalid: {0}... Valid object name: 1–1024 bytes (UTF-8 encoded). No null bytes. Leading/trailing whitespace is an error. Everything else is allowed including unicode.")]
    InvalidObjectName(String),

    #[error("Object is not valid: {0}")]
    InvalidObject(String, Option<String>),

    #[error("Provided integrity value does not match the generated integrity value.")]
    IntegrityValidationFailed(String),
}