use bytes::{Buf, BufMut, BytesMut};
use std::io;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::{FloeCommand, IglooCodable, IglooCommand};

const MAX_FRAME_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Error, Debug)]
pub enum IglooCodecError {
    #[error("Frame too large")]
    FrameTooLarge,
    #[error("Invalid message format")]
    InvalidMessage,
    #[error("Unknown command: {0}")]
    UnknownCommand(u8),
    #[error("Unknown component type: {0}")]
    UnknownComponent(u16),
    #[error("Invalid UTF-8")]
    InvalidUtf8,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid enum variant: {0}")]
    InvalidEnumVariant(String),
}

pub struct IglooCodec;

impl Decoder for IglooCodec {
    type Item = IglooCommand;
    type Error = IglooCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        // peek
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if length > MAX_FRAME_SIZE {
            return Err(IglooCodecError::FrameTooLarge);
        }

        if src.len() < 4 + length {
            // I NEED MORE BRUH
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        // got it! eat the frame
        src.advance(4);
        let mut frame = src.split_to(length).freeze();

        // speed decode
        IglooCommand::decode(&mut frame).map(Some)
    }
}

impl Encoder<FloeCommand> for IglooCodec {
    type Error = IglooCodecError;

    fn encode(&mut self, item: FloeCommand, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let estimated_len = item.est_encoded_len();
        dst.reserve(4 + estimated_len);

        // length placeholder
        let length_pos = dst.len();
        dst.put_u32_le(0);

        // encode msg
        let start_pos = dst.len();
        item.encode(dst)?;
        let actual_len = dst.len() - start_pos;

        // now we can set actual length
        let length_bytes = (actual_len as u32).to_le_bytes();
        dst[length_pos..length_pos + 4].copy_from_slice(&length_bytes);

        Ok(())
    }
}
