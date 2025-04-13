use crate::{low_level::PackFile, Error};
use bytes::{BufMut, BytesMut};
use std::fmt::Write;

/// The maximum length of a pkt-line's data component is 65516 bytes.
/// Implementations MUST NOT send pkt-line whose length exceeds 65520
/// (65516 bytes of payload + 4 bytes of length data).
///
/// <https://git-scm.com/docs/protocol-common#_pkt_line_format>
const MAX_DATA_LEN: usize = 65516;

/// A wrapper containing every possible type of message that can be sent to a Git client.
pub enum PktLine<'a> {
    /// General data sent to a client, generally a UTF-8 encoded string.
    Data(&'a [u8]),
    /// Similar to a data packet, but used during packfile sending to indicate this
    /// packet is a block of data by appending a byte containing the u8 `1`.
    SidebandData(PackFile<'a>),
    /// Similar to a data packet, but used during packfile sending to indicate this
    /// packet is a status message by appending a byte containing the u8 `2`.
    SidebandMsg(&'a [u8]),
    /// Indicates the end of a response.
    Flush,
    /// Separates sections of a response.
    Delimiter,
    /// Indicates the end of the response, allowing the client to send another request.
    ResponseEnd,
}

impl PktLine<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self, buf), err))]
    pub fn encode_to(&self, buf: &mut BytesMut) -> Result<(), Error> {
        match self {
            Self::Data(data) => {
                for chunk in data.chunks(MAX_DATA_LEN) {
                    write!(buf, "{:04x}", chunk.len() + 4)?;
                    buf.extend_from_slice(chunk);
                }
            }
            Self::SidebandData(packfile) => {
                // split the buf off so the cost of counting the bytes to put in the
                // data line prefix is just the cost of `unsplit` (an atomic decrement)
                let mut data_buf = buf.split_off(buf.len());

                packfile.encode_to(&mut data_buf)?;

                // write into the buf not the data buf so it's at the start of the msg
                if data_buf.len() + 5 <= MAX_DATA_LEN - 1 {
                    write!(buf, "{:04x}", data_buf.len() + 5)?;
                    buf.put_u8(1); // sideband, 1 = data
                    buf.unsplit(data_buf);
                } else {
                    for chunk in data_buf.chunks(MAX_DATA_LEN - 1) {
                        write!(buf, "{:04x}", chunk.len() + 5)?;
                        buf.put_u8(1); // sideband, 1 = data
                        buf.extend_from_slice(chunk);
                    }
                }
            }
            Self::SidebandMsg(msg) => {
                for chunk in msg.chunks(MAX_DATA_LEN - 1) {
                    write!(buf, "{:04x}", chunk.len() + 5)?;
                    buf.put_u8(2); // sideband, 2 = msg
                    buf.extend_from_slice(chunk);
                }
            }
            Self::Flush => buf.extend_from_slice(b"0000"),
            Self::Delimiter => buf.extend_from_slice(b"0001"),
            Self::ResponseEnd => buf.extend_from_slice(b"0002"),
        }

        Ok(())
    }
}

impl<'a> From<&'a str> for PktLine<'a> {
    fn from(val: &'a str) -> Self {
        PktLine::Data(val.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use crate::packet_line::MAX_DATA_LEN;
    use bytes::BytesMut;

    #[test]
    fn test_pkt_line() {
        let mut buffer = BytesMut::new();
        super::PktLine::from("agent=git/2.32.0\n")
            .encode_to(&mut buffer)
            .unwrap();
        assert_eq!(buffer.as_ref(), b"0015agent=git/2.32.0\n");
    }

    #[test]
    fn test_large_pkt_line() {
        let mut buffer = BytesMut::new();
        super::PktLine::from("a".repeat(70000).as_str())
            .encode_to(&mut buffer)
            .unwrap();
        assert_eq!(
            buffer.len(),
            70008,
            "should be two chunks each with a 4-byte len header"
        );

        // chunk 1
        assert_eq!(
            std::str::from_utf8(&buffer[..4]).unwrap(),
            format!("{:04x}", 4 + MAX_DATA_LEN)
        );
        assert!(
            &buffer[4..4 + MAX_DATA_LEN]
                .iter()
                .all(|b| char::from(*b) == 'a'),
            "data should be all 'a's"
        );

        // chunk 2
        assert_eq!(
            std::str::from_utf8(&buffer[4 + MAX_DATA_LEN..][..4]).unwrap(),
            format!("{:04x}", 4 + (70000 - MAX_DATA_LEN))
        );
        assert!(
            &buffer[4 + MAX_DATA_LEN + 4..]
                .iter()
                .all(|b| char::from(*b) == 'a'),
            "data should be all 'a's"
        );
    }
}
