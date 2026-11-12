//! Optional IPFS attachment helpers for chat.
#[cfg(feature = "ipfs")]
use std::path::Path;

#[cfg(feature = "ipfs")]
use gnostr_p2p::ipfs as p2p_ipfs;

#[cfg(feature = "ipfs")]
use crate::msg::Attachment;

#[cfg(feature = "ipfs")]
pub fn attach_file(path: &Path) -> anyhow::Result<Attachment> {
    let bytes = std::fs::read(path)?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment")
        .to_string();
    let mime = "application/octet-stream".to_string();

    let repo = std::env::temp_dir().join("gnostr-chat-ipfs");
    let _ = std::fs::create_dir_all(&repo);
    let state = p2p_ipfs::init_global(repo.to_str().unwrap_or("."), false);
    if !state.enabled {
        anyhow::bail!("ipfs node unavailable");
    }
    let cid = gnostr_p2p::ipfs_bridge::provide_block(&state, &bytes)
        .ok_or_else(|| anyhow::anyhow!("failed to add attachment to ipfs"))?;
    Ok(Attachment { cid, name, mime })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[cfg(feature = "ipfs")]
    #[ignore = "requires Go toolchain to build kubo-ffi"]
    fn attach_file_roundtrip() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"chat attachment").unwrap();
        let att = attach_file(f.path()).unwrap();
        assert!(!att.cid.is_empty());
        assert_eq!(att.name, f.path().file_name().unwrap().to_str().unwrap());
    }
}
