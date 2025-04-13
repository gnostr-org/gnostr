#![deny(clippy::pedantic)]
//! `packfile` is a simple library providing utilities to generate [Git Packfiles] in memory.
//!
//! Usage:
//!
//! ```rust
//! # use packfile::{high_level::GitRepository, low_level::PackFile};
//! #
//! let mut repo = GitRepository::default();
//! repo.insert(&["path", "to"], "file.txt", "hello world!".into()).unwrap();
//! let (_commit_hash, entries) =
//!     repo.commit("Linus Torvalds", "torvalds@example.com", "Some commit message").unwrap();
//!
//! let _packfile = PackFile::new(&entries);
//! ```
//!
//! The generated packfile can then be encoded within a [`SidebandData`] to send the data to a
//! client
//!
//! [Git Packfiles]: https://git-scm.com/book/en/v2/Git-Internals-Packfiles
//! [`SidebandData`]: crate::codec::Codec::SidebandData

#[cfg(feature = "tokio-util")]
pub mod codec;
mod error;
pub mod high_level;
pub mod low_level;
mod packet_line;
mod util;
pub mod network;
//pub use network;
pub use error::Error;
pub use packet_line::PktLine;

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use std::process::{Command, Stdio};
    use tempfile::TempDir;

    pub fn verify_pack_file(packed: Bytes) -> String {
        let scratch_dir = TempDir::new().unwrap();
        let packfile_path = scratch_dir.path().join("example.pack");

        std::fs::write(&packfile_path, packed).unwrap();

        let res = Command::new("git")
            .arg("index-pack")
            .arg(&packfile_path)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
        assert!(res.success());

        let command = Command::new("git")
            .arg("verify-pack")
            .arg("-v")
            .stdout(Stdio::piped())
            .arg(&packfile_path)
            .spawn()
            .unwrap();

        let out = command.wait_with_output().unwrap();
        assert!(out.status.success(), "git exited non-0");

        String::from_utf8(out.stdout).unwrap()
    }
}
