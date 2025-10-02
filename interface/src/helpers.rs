use borsh::BorshSerialize;
use bytes::{Buf, BytesMut};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
};
use tokio_util::codec::{Decoder, FramedRead};

use crate::{MAX_SUPPORTED_COMPONENT, WhatsUpIgloo};

pub const DEFAULT_COMMAND_SIZE: usize = 64;

pub type FloeWriterDefault = FloeWriter<BufWriter<OwnedWriteHalf>>;
pub type FloeReaderDefault = FramedRead<OwnedReadHalf, FloeCodec>;

pub struct FloeWriter<W: AsyncWriteExt + Unpin>(pub W);

pub async fn floe_init() -> Result<(FloeWriterDefault, FloeReaderDefault), std::io::Error> {
    let stream = UnixStream::connect("floe.sock").await.unwrap();

    let (reader, writer) = stream.into_split();
    let mut writer = FloeWriter(BufWriter::new(writer));
    let reader = FramedRead::new(reader, FloeCodec::new());

    writer
        .whats_up_igloo(&WhatsUpIgloo {
            max_supported_component: MAX_SUPPORTED_COMPONENT,
        })
        .await?;
    writer.flush().await?;

    Ok((writer, reader))
}

pub async fn floe_init_shared()
-> Result<(Arc<Mutex<FloeWriterDefault>>, Arc<Mutex<FloeReaderDefault>>), std::io::Error> {
    let (writer, reader) = floe_init().await?;
    Ok((Arc::new(Mutex::new(writer)), Arc::new(Mutex::new(reader))))
}

impl<W: AsyncWriteExt + Unpin> FloeWriter<W> {
    pub async fn write_no_payload(&mut self, cmd_id: u16) -> Result<(), std::io::Error> {
        self.write_varu32(1).await?;
        self.write_varu16(cmd_id).await?;
        Ok(())
    }

    pub async fn write_with_payload<P: BorshSerialize>(
        &mut self,
        cmd_id: u16,
        payload: &P,
    ) -> Result<(), std::io::Error> {
        let mut result = Vec::with_capacity(DEFAULT_COMMAND_SIZE);
        payload.serialize(&mut result)?;

        self.write_varu32(result.len() as u32 + 1).await?;
        self.write_varu16(cmd_id).await?;
        self.0.write_all(&result).await?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), std::io::Error> {
        self.0.flush().await
    }

    /// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
    pub async fn write_varu16(&mut self, value: u16) -> Result<(), std::io::Error> {
        if value <= 240 {
            self.0.write_u8(value as u8).await
        } else if value <= 2287 {
            let offset = value - 240;
            self.0.write_u8(241 + (offset >> 8) as u8).await?;
            self.0.write_u8((offset & 0xFF) as u8).await
        } else {
            let offset = value - 2288;
            self.0.write_u8(249).await?;
            self.0.write_u8((offset >> 8) as u8).await?;
            self.0.write_u8((offset & 0xFF) as u8).await
        }
    }

    /// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
    pub async fn write_varu32(&mut self, value: u32) -> Result<(), std::io::Error> {
        if value <= 240 {
            self.0.write_u8(value as u8).await
        } else if value <= 2287 {
            let offset = value - 240;
            self.0.write_u8(241 + (offset >> 8) as u8).await?;
            self.0.write_u8((offset & 0xFF) as u8).await
        } else if value <= 67823 {
            let offset = value - 2288;
            self.0.write_u8(249).await?;
            self.0.write_u8((offset >> 8) as u8).await?;
            self.0.write_u8((offset & 0xFF) as u8).await
        } else if value <= 16777215 {
            let offset = value - 67824;
            self.0.write_u8(250).await?;
            self.0.write_u8((offset >> 16) as u8).await?;
            self.0.write_u8((offset >> 8) as u8).await?;
            self.0.write_u8((offset & 0xFF) as u8).await
        } else {
            let offset = value - 16777216;
            self.0.write_u8(251).await?;
            self.0.write_u8((offset >> 24) as u8).await?;
            self.0.write_u8((offset >> 16) as u8).await?;
            self.0.write_u8((offset >> 8) as u8).await?;
            self.0.write_u8((offset & 0xFF) as u8).await
        }
    }
}

pub struct FloeCodec {
    state: DecodeState,
}

#[derive(Debug)]
enum DecodeState {
    ReadingLength,
    ReadingCmdId { total_length: u32 },
    ReadingPayload { cmd_id: u16, payload_length: usize },
}

impl FloeCodec {
    pub fn new() -> Self {
        FloeCodec {
            state: DecodeState::ReadingLength,
        }
    }

    /// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
    fn decode_varu16(src: &mut BytesMut) -> Option<u16> {
        if src.is_empty() {
            return None;
        }

        let first_byte = src[0];

        if first_byte <= 240 {
            src.advance(1);
            Some(first_byte as u16)
        } else if first_byte <= 248 {
            if src.len() < 2 {
                return None;
            }
            let high = (first_byte - 241) as u16;
            let low = src[1] as u16;
            src.advance(2);
            Some(240 + (high << 8) + low)
        } else if first_byte == 249 {
            if src.len() < 3 {
                return None;
            }
            let high = src[1] as u16;
            let low = src[2] as u16;
            src.advance(3);
            Some(2288 + (high << 8) + low)
        } else {
            src.advance(1);
            Some(0) // TODO return error
        }
    }

    /// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
    fn decode_varu32(src: &mut BytesMut) -> Option<u32> {
        if src.is_empty() {
            return None;
        }

        let first_byte = src[0];

        if first_byte <= 240 {
            src.advance(1);
            Some(first_byte as u32)
        } else if first_byte <= 248 {
            if src.len() < 2 {
                return None;
            }
            let high = (first_byte - 241) as u32;
            let low = src[1] as u32;
            src.advance(2);
            Some(240 + (high << 8) + low)
        } else if first_byte == 249 {
            if src.len() < 3 {
                return None;
            }
            let high = src[1] as u32;
            let low = src[2] as u32;
            src.advance(3);
            Some(2288 + (high << 8) + low)
        } else if first_byte == 250 {
            if src.len() < 4 {
                return None;
            }
            let b1 = src[1] as u32;
            let b2 = src[2] as u32;
            let b3 = src[3] as u32;
            src.advance(4);
            Some(67824 + (b1 << 16) + (b2 << 8) + b3)
        } else if first_byte == 251 {
            if src.len() < 5 {
                return None;
            }
            let b1 = src[1] as u32;
            let b2 = src[2] as u32;
            let b3 = src[3] as u32;
            let b4 = src[4] as u32;
            src.advance(5);
            Some(16777216 + (b1 << 24) + (b2 << 16) + (b3 << 8) + b4)
        } else {
            src.advance(1);
            Some(0) // TODO return error
        }
    }

    fn varu16_size(value: u16) -> usize {
        if value <= 240 {
            1
        } else if value <= 2287 {
            2
        } else {
            3
        }
    }
}

impl Decoder for FloeCodec {
    type Item = (u16, Vec<u8>);
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match &mut self.state {
                DecodeState::ReadingLength => {
                    if let Some(total_length) = Self::decode_varu32(src) {
                        self.state = DecodeState::ReadingCmdId { total_length };
                    } else {
                        return Ok(None);
                    }
                }
                DecodeState::ReadingCmdId { total_length } => {
                    let mut peek_buf = src.clone();
                    if let Some(cmd_id) = Self::decode_varu16(&mut peek_buf) {
                        Self::decode_varu16(src);

                        let cmd_id_size = Self::varu16_size(cmd_id);
                        let payload_length = (*total_length as usize).saturating_sub(cmd_id_size);

                        self.state = DecodeState::ReadingPayload {
                            cmd_id,
                            payload_length,
                        };
                    } else {
                        return Ok(None);
                    }
                }
                DecodeState::ReadingPayload {
                    cmd_id,
                    payload_length,
                } => {
                    if src.len() >= *payload_length {
                        let payload = src.split_to(*payload_length).to_vec();
                        let result = (*cmd_id, payload);

                        self.state = DecodeState::ReadingLength;

                        return Ok(Some(result));
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }
}
