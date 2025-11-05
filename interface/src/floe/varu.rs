use bytes::{Buf, BytesMut};
use tokio::io::AsyncWriteExt;

use crate::floe::{FloeWriter, codec::FloeCodec};

impl FloeCodec {
    /// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
    pub fn decode_varu16(src: &mut BytesMut) -> Option<u16> {
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
    pub fn decode_varu32(src: &mut BytesMut) -> Option<u32> {
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

    pub fn varu16_size(value: u16) -> usize {
        if value <= 240 {
            1
        } else if value <= 2287 {
            2
        } else {
            3
        }
    }
}

impl<W: AsyncWriteExt + Unpin> FloeWriter<W> {
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
