use std::ops::RangeInclusive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to write formatted string to buffer: {0}")]
    BufferWrite(#[from] std::fmt::Error),
    #[error("{0} is not a directory")]
    NotDirectory(&'static str),
    #[cfg(feature = "tokio-util")]
    #[error("Failed to parse utf-8 encoded prefix: {0}")]
    ParseLengthBytes(std::str::Utf8Error),
    #[cfg(feature = "tokio-util")]
    #[error("Failed to parse length from hex string: {0}")]
    ParseLengthAsHex(std::num::ParseIntError),
    #[error("Failed to write bytes to compress to zlib: {0}")]
    CompressWrite(std::io::Error),
    #[error("Failed to compress packfile with zlib: {0}")]
    Compress(std::io::Error),
    #[error("Failed to encode tree hash to hex: {0}")]
    EncodeTreeHash(hex::FromHexError),
    #[error("Entries in packfile exceeds a u32: {0}")]
    EntriesExceedsU32(std::num::TryFromIntError),
    #[error("Packet length is not in the range {0:?} as defined by the spec, got {1}")]
    PacketLengthExceedsSpec(RangeInclusive<usize>, usize),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
