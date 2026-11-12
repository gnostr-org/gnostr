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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cid_as_str_returns_inner_string() {
        // Arrange
        let cid = Cid("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string());

        // Act
        let s = cid.as_str();

        // Assert
        assert_eq!(
            s,
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        );
    }

    #[test]
    fn cid_display_delegates_to_inner_string() {
        // Arrange
        let cid = Cid("QmExample123".to_string());

        // Act
        let formatted = format!("{cid}");

        // Assert
        assert_eq!(formatted, "QmExample123");
    }

    #[test]
    fn cid_traits_behave() {
        // Arrange
        let a = Cid("bafyA".to_string());
        let b = a.clone();
        let c = Cid("bafyB".to_string());

        // Act / Assert
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a < c); // Ord derives from inner String
        let mut set = std::collections::HashSet::new();
        set.insert(a.clone());
        set.insert(b);
        set.insert(c);
        assert_eq!(set.len(), 2); // Hash/Eq agree with PartialEq
    }

    #[test]
    fn ipfs_content_equality() {
        // Arrange
        let content = IpfsContent {
            cid: Cid("bafyA".to_string()),
            name: Some("photo.jpg".to_string()),
            mime: Some("image/jpeg".to_string()),
        };

        // Act
        let same = IpfsContent {
            cid: Cid("bafyA".to_string()),
            name: Some("photo.jpg".to_string()),
            mime: Some("image/jpeg".to_string()),
        };
        let different = IpfsContent {
            cid: Cid("bafyB".to_string()),
            name: None,
            mime: None,
        };

        // Assert
        assert_eq!(content, same);
        assert_ne!(content, different);
    }
}
