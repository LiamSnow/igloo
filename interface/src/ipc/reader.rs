use crate::ipc::IglooMessage;
use bincode::config;
use futures_core::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::unix::OwnedReadHalf;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

#[derive(Debug)]
pub struct IReader {
    framed: FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
}

#[derive(thiserror::Error, Debug)]
pub enum IReaderError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Bincode Decode Error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
}

impl IReader {
    pub fn new(reader: OwnedReadHalf) -> Self {
        let codec = LengthDelimitedCodec::builder()
            .big_endian()
            .length_field_length(4)
            .new_codec();

        Self {
            framed: FramedRead::new(reader, codec),
        }
    }

    pub fn get_ref(&self) -> &OwnedReadHalf {
        self.framed.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut OwnedReadHalf {
        self.framed.get_mut()
    }

    pub fn into_inner(self) -> OwnedReadHalf {
        self.framed.into_inner()
    }
}

impl Stream for IReader {
    type Item = Result<IglooMessage, IReaderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.framed).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                match bincode::decode_from_slice(&bytes, config::standard()) {
                    Ok((message, _)) => Poll::Ready(Some(Ok(message))),
                    Err(e) => Poll::Ready(Some(Err(IReaderError::Decode(e)))),
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(IReaderError::Io(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
