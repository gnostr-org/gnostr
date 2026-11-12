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

#[cfg(test)]
mod test {
    use super::*;

    async fn write_temp_file(contents: &[u8]) -> tempfile::NamedTempFile {
        use tokio::io::AsyncWriteExt;
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut f = File::from_std(file.reopen().unwrap());
        f.write_all(contents).await.unwrap();
        f.flush().await.unwrap();
        file
    }

    #[tokio::test]
    async fn read_slice_reads_exact_range() {
        // Arrange
        let file = write_temp_file(b"hello world").await;
        let reader = FileSliceReader::new(file.path());

        // Act
        let buf = reader.read_slice(0, 5).await.unwrap();

        // Assert
        assert_eq!(buf, b"hello");
    }

    #[tokio::test]
    async fn read_slice_honours_offset() {
        // Arrange
        let file = write_temp_file(b"hello world").await;
        let reader = FileSliceReader::new(file.path());

        // Act
        let buf = reader.read_slice(6, 5).await.unwrap();

        // Assert
        assert_eq!(buf, b"world");
    }

    #[tokio::test]
    async fn read_slice_past_eof_returns_error() {
        // Arrange
        let file = write_temp_file(b"short").await;
        let reader = FileSliceReader::new(file.path());

        // Act
        let result = reader.read_slice(0, 1024).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_slice_missing_file_returns_error() {
        // Arrange
        let reader = FileSliceReader::new("/nonexistent/gnostr-file-slice-test.bin");

        // Act
        let result = reader.read_slice(0, 1).await;

        // Assert
        assert!(result.is_err());
    }
}
