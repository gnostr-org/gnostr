//! `gnostr-p2p` is the Rust crate that owns the P2P package namespace.
//!
//! The browser-side pure JavaScript implementation lives under `src/js/`.

use std::path::PathBuf;

/// Crate name.
pub const PACKAGE_NAME: &str = "gnostr-p2p";

/// Return the absolute path to the bundled JavaScript source tree.
pub fn js_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_js_source_dir() {
        assert!(js_source_dir().ends_with("p2p/src/js"));
    }
}
