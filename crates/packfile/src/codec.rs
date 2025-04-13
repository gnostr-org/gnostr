//! A [`tokio_util::codec`] implementation for the [Git wire format].
//!
//! [Git wire format]: https://git-scm.com/docs/protocol-v2

#![allow(clippy::module_name_repetitions)]

use std::ops::RangeInclusive;

use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec;

use crate::{packet_line::PktLine, Error};

const ALLOWED_PACKET_LENGTH: RangeInclusive<usize> = 4..=65520;

pub struct Encoder;

impl codec::Encoder<PktLine<'_>> for Encoder {
    type Error = Error;

    fn encode(&mut self, item: PktLine<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode_to(dst)?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GitCommand {
    pub command: Bytes,
    pub metadata: Vec<Bytes>,
}

#[derive(Default)]
pub struct GitCodec {
    command: GitCommand,
}

impl codec::Decoder for GitCodec {
    type Item = GitCommand;
    type Error = Error;

    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, src), err))]
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if src.len() < 4 {
                return Ok(None);
            }

            let mut length_bytes = [0_u8; 4];
            length_bytes.copy_from_slice(&src[..4]);
            let length = u16::from_str_radix(
                std::str::from_utf8(&length_bytes).map_err(Error::ParseLengthBytes)?,
                16,
            )
            .map_err(Error::ParseLengthAsHex)? as usize;

            if length == 0 {
                // flush
                src.advance(4);
                return Ok(Some(std::mem::take(&mut self.command)));
            } else if length == 1 || length == 2 {
                src.advance(4);
                continue;
            } else if !ALLOWED_PACKET_LENGTH.contains(&length) {
                return Err(Error::PacketLengthExceedsSpec(
                    ALLOWED_PACKET_LENGTH,
                    length,
                ));
            }

            // not enough bytes in the buffer yet, ask for more
            if src.len() < length {
                src.reserve(length - src.len());
                return Ok(None);
            }

            // length is inclusive of the 4 bytes that makes up itself
            let mut data = src.split_to(length).freeze();
            data.advance(4);

            // strip newlines for conformity
            if data.ends_with(b"\n") {
                data.truncate(data.len() - 1);
            }

            if self.command.command.is_empty() {
                self.command.command = data;
            } else {
                self.command.metadata.push(data);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PktLine;
    use bytes::{Bytes, BytesMut};
    use std::fmt::Write;
    use tokio_util::codec::{Decoder, Encoder};

    #[test]
    fn encode() {
        let mut bytes = BytesMut::new();
        super::Encoder
            .encode(PktLine::Data(&[1, 2, 3, 4]), &mut bytes)
            .unwrap();

        assert_eq!(bytes.to_vec(), b"0008\x01\x02\x03\x04");
    }

    #[test]
    fn decode() {
        let mut codec = super::GitCodec::default();

        let mut bytes = BytesMut::new();

        bytes.write_str("0015agent=git/2.32.0").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res, None);

        bytes.write_char('\n').unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res, None);

        bytes.write_str("0000").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(
            res,
            Some(super::GitCommand {
                command: Bytes::from_static(b"agent=git/2.32.0"),
                metadata: vec![],
            })
        );

        bytes.write_str("0000").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(
            res,
            Some(super::GitCommand {
                command: Bytes::new(),
                metadata: vec![],
            })
        );

        bytes.write_str("0002").unwrap();
        bytes.write_str("0005a").unwrap();
        bytes.write_str("0001").unwrap();
        bytes.write_str("0005b").unwrap();
        bytes.write_str("0000").unwrap();

        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(
            res,
            Some(super::GitCommand {
                command: Bytes::from_static(b"a"),
                metadata: vec![Bytes::from_static(b"b")],
            })
        );
    }
}
