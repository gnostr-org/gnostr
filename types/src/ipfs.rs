//! IPFS content identifiers and references shared across crates.

/// A content identifier (CID) string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cid(pub String);

impl Cid {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A reference to content stored in IPFS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpfsContent {
    pub cid: Cid,
    pub name: Option<String>,
    pub mime: Option<String>,
}
