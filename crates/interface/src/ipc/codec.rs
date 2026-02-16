//! See [Tokio's implementation](https://docs.rs/tokio-util/latest/src/tokio_util/codec/length_delimited.rs.html)

use bytes::{Buf, BufMut, BytesMut};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io;
use std::marker::PhantomData;
use tokio_util::codec::{Decoder, Encoder};

/// A codec for JSON frames delimited by a 4-byte little-endian length prefix.
///
/// Frame format:
/// ```text
/// +---------------+--------------------------------+
/// | len: u32 (LE) |     JSON payload (UTF-8)       |
/// +---------------+--------------------------------+
/// ```
#[derive(Debug, Clone)]
pub struct LengthDelimitedJSONCodec<T> {
    _phantom: PhantomData<T>,
}

impl<T> LengthDelimitedJSONCodec<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for LengthDelimitedJSONCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Decoder for LengthDelimitedJSONCodec<T>
where
    T: DeserializeOwned,
{
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if src.len() < 4 + length {
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        src.advance(4);

        let data = src.split_to(length);
        let item = serde_json::from_slice(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Some(item))
    }
}

impl<T> Encoder<T> for LengthDelimitedJSONCodec<T>
where
    T: Serialize,
{
    type Error = io::Error;

    fn encode(&mut self, item: T, dst: &mut BytesMut) -> io::Result<()> {
        let json_bytes =
            serde_json::to_vec(&item).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        dst.reserve(4 + json_bytes.len());
        dst.put_u32_le(json_bytes.len() as u32);
        dst.extend_from_slice(&json_bytes);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestMessage {
        id: u32,
        text: String,
    }

    #[test]
    fn test_encode_decode() {
        let mut codec = LengthDelimitedJSONCodec::<TestMessage>::new();
        let mut buffer = BytesMut::new();

        let msg = TestMessage {
            id: 42,
            text: "hello".to_string(),
        };
        codec.encode(msg.clone(), &mut buffer).unwrap();

        assert!(buffer.len() >= 4);
        let length = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(buffer.len(), 4 + length as usize);

        let decoded = codec.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(decoded, msg);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_partial_frame() {
        let mut codec = LengthDelimitedJSONCodec::<TestMessage>::new();
        let mut buffer = BytesMut::new();

        buffer.extend_from_slice(&[0x00, 0x01]);
        assert!(codec.decode(&mut buffer).unwrap().is_none());

        buffer.extend_from_slice(&[0x00, 0x00]);
        assert!(codec.decode(&mut buffer).unwrap().is_none());

        buffer.extend_from_slice(b"{");
        let result = codec.decode(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_frames() {
        let mut encoder = LengthDelimitedJSONCodec::<TestMessage>::new();
        let mut decoder = LengthDelimitedJSONCodec::<TestMessage>::new();
        let mut buffer = BytesMut::new();

        let msg1 = TestMessage {
            id: 1,
            text: "first".to_string(),
        };
        let msg2 = TestMessage {
            id: 2,
            text: "second".to_string(),
        };

        encoder.encode(msg1.clone(), &mut buffer).unwrap();
        encoder.encode(msg2.clone(), &mut buffer).unwrap();

        let decoded1 = decoder.decode(&mut buffer).unwrap().unwrap();
        let decoded2 = decoder.decode(&mut buffer).unwrap().unwrap();

        assert_eq!(decoded1, msg1);
        assert_eq!(decoded2, msg2);
        assert!(buffer.is_empty());
    }
}
