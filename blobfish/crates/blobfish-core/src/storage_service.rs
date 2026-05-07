use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncRead;
use crate::errors::AppError;
use crate::models::config::{Config};
use crate::models::object::ChunkDescriptor;

pub struct StorageService{
    full_dir: PathBuf
}
impl StorageService {
    pub fn new(config: Config) -> Self {
        Self{
            full_dir: config.storage.chunk_dir.clone()
        }
    }
    pub async fn write_to_disk(&self, mut reader: impl AsyncRead + Unpin, key: &str) -> anyhow::Result<Vec<ChunkDescriptor>>{
        let mut result: Vec<ChunkDescriptor> = vec![];

        let mut working = ChunkDescriptor::new();

        let mut path = self.full_dir.clone();
        path.push(working.chunk_id.to_string());

        let mut file = File::create(path).await
            .map_err(|_| anyhow::Error::from(AppError::InvalidObject(key.to_string())))?;

        let content_length = tokio::io::copy(&mut reader, &mut file).await
            .map_err(|_| anyhow::Error::from(AppError::InvalidObject(key.to_string())))?;

        working.size_bytes = content_length;

        result.push(working.clone());

        Ok(result)
    }
    pub async fn read_from_disk(&self, chunks: Vec<ChunkDescriptor>, key: &str) -> anyhow::Result<File>{
        let working = chunks.first().unwrap();
        let mut path = self.full_dir.clone();
        path.push(working.chunk_id.to_string());

        Ok(File::open(&path).await
            .map_err(|_| AppError::ObjectNotFound(key.to_string()))?)
    }
}