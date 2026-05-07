use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectKey{
    //Constraints: 1–1024 bytes (UTF-8 encoded). No null bytes. Leading/trailing whitespace is an error. Everything else is allowed including unicode.
    pub key: String,
    pub bucket: String,
    pub key_id: Uuid,
    pub current_version: Uuid,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ObjectKey{
    pub fn new(key: &str, bucket: &str, version_id: Uuid) -> Self{
        Self{
            key: key.to_string(),
            bucket: bucket.to_string(),
            key_id: Uuid::new_v4(),
            current_version: version_id,
            deleted_at: None,
        }
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn is_valid(&self) -> Result<bool, AppError> {
        let s = &self.key;
        let byte_len = s.len();

        if byte_len == 0 {
            return Err(AppError::InvalidObjectName("empty".to_string()));
        }

        if byte_len > 1024 {
            return Err(AppError::InvalidObjectName("too long".to_string()));
        }

        if s.contains('\0') {
            return Err(AppError::InvalidObjectName("contains invalid characters".to_string()));
        }

        // Check leading whitespace via the first char
        if s.chars().next().is_some_and(|c| c.is_whitespace()) {
            return Err(AppError::InvalidObjectName("contains invalid characters".to_string()));
        }

        // Check trailing whitespace via the last char
        if s.chars().last().is_some_and(|c| c.is_whitespace()) {
            return Err(AppError::InvalidObjectName("contains invalid characters".to_string()));
        }

        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectVersion {
    pub version_id: Uuid,
    pub key: Uuid,
    pub size_bytes: u64,
    pub content_type: Option<String>,
    pub checksum_sha256: String,
    pub created_at: DateTime<Utc>,
    pub chunks: Vec<Uuid>,
}

impl ObjectVersion{
    pub fn new(key_id: Uuid, content_type: &str, chunks: Vec<ChunkDescriptor>, version_id: Uuid) -> Self{
        Self{
            version_id,
            key: key_id,
            size_bytes: chunks.iter().map(|item| item.size_bytes).sum(),
            content_type: Some(content_type.to_string()),
            checksum_sha256: "".to_string(),
            created_at: Utc::now(),
            chunks: chunks.iter().map(|u| u.chunk_id).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDescriptor {
    pub chunk_id: Uuid,
    pub ordinal: u32,
    pub offset: u64,
    pub size_bytes: u64,
    pub checksum_sha256: String,
}

impl ChunkDescriptor{
    pub fn new() -> Self{
        Self{
            chunk_id: Uuid::new_v4(),
            ordinal: 0,
            offset: 0,
            size_bytes: 0,
            checksum_sha256: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // helper to reduce boilerplate
    fn make_key(key: &str) -> ObjectKey {
        ObjectKey::new(key, "test-bucket", Uuid::new_v4())
    }

    // --- valid cases ---

    #[test]
    fn valid_simple_ascii() {
        assert!(make_key("my-object.txt").is_valid().unwrap());
    }

    #[test]
    fn valid_single_byte() {
        assert!(make_key("a").is_valid().unwrap());
    }

    #[test]
    fn valid_exactly_1024_bytes() {
        let key = "a".repeat(1024);
        assert!(make_key(&key).is_valid().unwrap());
    }

    #[test]
    fn valid_unicode() {
        assert!(make_key("日本語/object.txt").is_valid().unwrap());
    }

    #[test]
    fn valid_unicode_emoji() {
        assert!(make_key("folder/🦀/file").is_valid().unwrap());
    }

    #[test]
    fn valid_internal_whitespace() {
        // whitespace in the middle is allowed
        assert!(make_key("my object name").is_valid().unwrap());
    }

    #[test]
    fn valid_internal_newline() {
        assert!(make_key("my\nobject").is_valid().unwrap());
    }

    #[test]
    fn valid_slashes_and_special_chars() {
        assert!(make_key("folder/sub-folder/file_name.v2.tar.gz").is_valid().unwrap());
    }

    #[test]
    fn valid_only_special_chars() {
        assert!(make_key("!@#$%^&*()").is_valid().unwrap());
    }

    // --- empty ---

    #[test]
    fn invalid_empty_string() {
        let err = make_key("").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "empty"));
    }

    // --- too long ---

    #[test]
    fn invalid_1025_bytes() {
        let key = "a".repeat(1025);
        let err = make_key(&key).is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "too long"));
    }

    #[test]
    fn invalid_very_long_key() {
        let key = "a".repeat(9999);
        let err = make_key(&key).is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "too long"));
    }

    // byte length vs char length - a 4-byte emoji repeated 257 times = 1028 bytes
    #[test]
    fn invalid_multibyte_chars_exceeding_byte_limit() {
        let key = "🦀".repeat(257); // 257 * 4 = 1028 bytes
        let err = make_key(&key).is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "too long"));
    }

    // --- null bytes ---

    #[test]
    fn invalid_null_byte() {
        let err = make_key("object\0name").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_only_null_byte() {
        let err = make_key("\0").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    // --- leading whitespace ---

    #[test]
    fn invalid_leading_space() {
        let err = make_key(" object").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_leading_tab() {
        let err = make_key("\tobject").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_leading_newline() {
        let err = make_key("\nobject").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_leading_unicode_whitespace() {
        // U+00A0 non-breaking space
        let err = make_key("\u{00A0}object").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    // --- trailing whitespace ---

    #[test]
    fn invalid_trailing_space() {
        let err = make_key("object ").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_trailing_tab() {
        let err = make_key("object\t").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_trailing_newline() {
        let err = make_key("object\n").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_trailing_unicode_whitespace() {
        let err = make_key("object\u{00A0}").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    // --- only whitespace (hits leading check first) ---

    #[test]
    fn invalid_only_spaces() {
        let err = make_key("   ").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    #[test]
    fn invalid_single_space() {
        let err = make_key(" ").is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "contains invalid characters"));
    }

    // --- boundary: exactly 1 and 1024 bytes with unicode ---

    #[test]
    fn valid_single_unicode_char() {
        // é is 2 bytes — still valid, above zero
        assert!(make_key("é").is_valid().unwrap());
    }

    #[test]
    fn valid_1024_bytes_unicode() {
        // "é" is 2 bytes, so 512 repetitions = exactly 1024 bytes
        let key = "é".repeat(512);
        assert_eq!(key.len(), 1024);
        assert!(make_key(&key).is_valid().unwrap());
    }

    #[test]
    fn invalid_1025_bytes_unicode() {
        // 512 * "é" (1024 bytes) + "a" (1 byte) = 1025 bytes
        let key = format!("{}{}", "é".repeat(512), "a");
        assert_eq!(key.len(), 1025);
        let err = make_key(&key).is_valid().unwrap_err();
        assert!(matches!(err, AppError::InvalidObjectName(msg) if msg == "too long"));
    }
}