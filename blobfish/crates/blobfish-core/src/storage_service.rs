use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncRead;
use crate::errors::AppError;
use crate::models::config::{Config};
use crate::models::object::ChunkDescriptor;
use sha2::{Sha256, Digest};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io;

pub struct StorageService{
    full_dir: PathBuf,
    perm_dir: PathBuf
}
impl StorageService {
    pub fn new(config: Config) -> Self {
        let full_dir = config.storage.chunk_dir.clone();
        let perm_dir = config.storage.perm_dir.clone();
        std::fs::create_dir_all(&full_dir).unwrap_or_else(|e| {
            panic!(
                "failed to create storage chunk_dir '{}': {}",
                full_dir.display(),
                e
            )
        });
        std::fs::create_dir_all(&perm_dir).unwrap_or_else(|e| {
            panic!(
                "failed to create storage perm_dir '{}': {}",
                perm_dir.display(),
                e
            )
        });
        Self{
            full_dir,
            perm_dir
        }
    }
    pub async fn write_to_disk(&self, reader: impl AsyncRead + Unpin, key: &str) -> anyhow::Result<Vec<ChunkDescriptor>> {
        let mut result: Vec<ChunkDescriptor> = vec![];
        let mut working = ChunkDescriptor::new();

        // Write to staging with temp name
        let mut staging_path = self.full_dir.clone();
        staging_path.push(working.temp_id.to_string());

        let mut file = File::create(&staging_path).await
            .map_err(|e| anyhow::Error::from(AppError::InvalidObject(key.to_string(), Option::from(e.to_string()))))?;

        let mut hashing_reader = HashingReader::new(reader);

        let content_length = tokio::io::copy(&mut hashing_reader, &mut file).await
            .map_err(|e| anyhow::Error::from(AppError::InvalidObject(key.to_string(), Option::from(e.to_string()))))?;

        // Finalize hash and build permanent path
        let hash_bytes = hashing_reader.finalize();
        let hash_hex = hex::encode(&hash_bytes);

        let mut perm_path = self.perm_dir.clone();
        perm_path.push(&hash_hex);

        if !perm_path.exists(){
            // Atomically move from staging -> permanent
            tokio::fs::rename(&staging_path, &perm_path).await
                .map_err(|e| anyhow::Error::from(AppError::InvalidObject(key.to_string(), Option::from(e.to_string()))))?;
        }

        working.set(hash_hex.into(), hash_bytes, content_length);

        result.push(working.clone());
        Ok(result)
    }
    pub async fn read_from_disk(&self, chunks: Vec<ChunkDescriptor>, key: &str) -> anyhow::Result<File>{
        let working = chunks.first().ok_or_else(|| {
            anyhow::Error::from(AppError::InvalidObject(
                key.to_string(),
                Some("object has no chunks".to_string()),
            ))
        })?;
        let mut path = self.perm_dir.clone();
        path.push(working.chunk_id.to_string());

        Ok(File::open(&path).await
            .map_err(|_| AppError::ObjectNotFound(key.to_string()))?)
    }
}

struct HashingReader<R> {
    inner: R,
    hasher: Sha256,
}

impl<R> HashingReader<R> {
    fn new(inner: R) -> Self {
        Self { inner, hasher: Sha256::new() }
    }

    fn finalize(self) -> Vec<u8> {
        self.hasher.finalize().to_vec()
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for HashingReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &result {
            let new_bytes = &buf.filled()[before..];
            self.hasher.update(new_bytes);
        }
        result
    }
}