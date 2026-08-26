use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
pub async fn canonicalize_path(path: &Path) -> io::Result<PathBuf> {
    let absolute_path = if path.is_relative() {
        let current_dir = env::current_dir()?;
        current_dir.join(path)
    } else {
        path.to_path_buf()
    };
    fs::canonicalize(absolute_path)
}
