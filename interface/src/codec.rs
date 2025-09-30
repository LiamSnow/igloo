use borsh::{BorshDeserialize, BorshSerialize};
use bytes::{Buf, BufMut, BytesMut};
use std::marker::PhantomData;
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

use crate::{FloeCommand, IglooCommand};

const MAX_FRAME_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("IO: `{0}`")]
    IO(#[from] std::io::Error),
    #[error("Frame too large: {0} bytes")]
    FrameTooLarge(usize),
}

pub type FloeCodec = FrameCodec<IglooCommand, FloeCommand>;
pub type IglooCodec = FrameCodec<FloeCommand, IglooCommand>;

pub struct FrameCodec<D, E> {
    _decode: PhantomData<D>,
    _encode: PhantomData<E>,
}

impl<D, E> Default for FrameCodec<D, E> {
    fn default() -> Self {
        Self {
            _decode: PhantomData,
            _encode: PhantomData,
        }
    }
}

impl<D, E> FrameCodec<D, E> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D, E> Decoder for FrameCodec<D, E>
where
    D: BorshDeserialize,
{
    type Item = D;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }

        // peek at length
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if length > MAX_FRAME_SIZE {
            return Err(CodecError::FrameTooLarge(length));
        }

        if src.len() < 4 + length {
            // need more data
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        // good -> take it
        src.advance(4);
        let frame = src.split_to(length);

        // decode
        let command = borsh::from_slice(&frame[..])?;

        Ok(Some(command))
    }
}

impl<D, E> Encoder<E> for FrameCodec<D, E>
where
    E: BorshSerialize,
{
    type Error = CodecError;

    fn encode(&mut self, item: E, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let encoded = borsh::to_vec(&item)?;

        if encoded.len() > MAX_FRAME_SIZE {
            return Err(CodecError::FrameTooLarge(encoded.len()));
        }

        dst.reserve(4 + encoded.len());
        dst.put_u32_le(encoded.len() as u32);
        dst.put_slice(&encoded);

        Ok(())
    }
}
