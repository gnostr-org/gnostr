// Dimensions for an image

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

/// Dimensions for an image (width and height).
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct ImageDimensions {
    /// The width of the image.
    pub width: u64,
    /// The height of the image.
    pub height: u64,
}
