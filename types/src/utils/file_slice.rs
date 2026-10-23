use std::path::PathBuf;

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

/// Reads byte ranges from a file.
pub struct FileSliceReader {
    file_path: PathBuf,
}

impl FileSliceReader {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }

    pub async fn read_slice(&self, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(&self.file_path).await?;
        file.seek(SeekFrom::Start(offset)).await?;
        let mut buf = vec![0; len];
        file.read_exact(&mut buf).await?;
        Ok(buf)
    }
}