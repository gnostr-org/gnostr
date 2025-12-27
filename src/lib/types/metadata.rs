use super::versioned::metadata::MetadataV1;

/// Metadata about a user
///
/// Note: the value is an Option because some real-world data has been found to
/// contain JSON nulls as values, and we don't want deserialization of those
/// events to fail. We treat these in our get() function the same as if the key
/// did not exist.

pub(super) const DEFAULT_AVATAR: &str = "https://avatars.githubusercontent.com/u/135379339?s=400&u=11cb72cccbc2b13252867099546074c50caef1ae&v=4";
pub(super) const DEFAULT_BANNER: &str = "https://raw.githubusercontent.com/gnostr-org/gnostr-icons/refs/heads/master/banner/1024x341.png";

pub type Metadata = MetadataV1;
