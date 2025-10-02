use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt},
    net::TcpStream,
};

#[async_trait]
pub trait Varu32: AsyncReadExt {
    async fn read_varu32(&mut self) -> io::Result<u32>;
}

#[async_trait]
impl Varu32 for TcpStream {
    /// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
    async fn read_varu32(&mut self) -> io::Result<u32> {
        let first_byte = self.read_u8().await?;

        if first_byte <= 240 {
            Ok(first_byte as u32)
        } else if first_byte <= 248 {
            let low = self.read_u8().await?;
            let high = (first_byte - 241) as u32;
            Ok(240 + (high << 8) + low as u32)
        } else if first_byte == 249 {
            let mut buf = [0u8; 2];
            self.read_exact(&mut buf).await?;
            let high = buf[0] as u32;
            let low = buf[1] as u32;
            Ok(2288 + (high << 8) + low)
        } else if first_byte == 250 {
            let mut buf = [0u8; 3];
            self.read_exact(&mut buf).await?;
            let b1 = buf[0] as u32;
            let b2 = buf[1] as u32;
            let b3 = buf[2] as u32;
            Ok(67824 + (b1 << 16) + (b2 << 8) + b3)
        } else if first_byte == 251 {
            let mut buf = [0u8; 4];
            self.read_exact(&mut buf).await?;
            let b1 = buf[0] as u32;
            let b2 = buf[1] as u32;
            let b3 = buf[2] as u32;
            let b4 = buf[3] as u32;
            Ok(16777216 + (b1 << 24) + (b2 << 16) + (b3 << 8) + b4)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid varint first byte",
            ))
        }
    }
}

/// [Docs](https://sqlite.org/src4/doc/trunk/www/varint.wiki)
pub fn varu32_to_bytes(value: u32) -> BytesMut {
    let mut bytes = BytesMut::new();

    if value <= 240 {
        bytes.put_u8(value as u8);
    } else if value <= 2287 {
        let offset = value - 240;
        bytes.put_u8(241 + (offset >> 8) as u8);
        bytes.put_u8((offset & 0xFF) as u8);
    } else if value <= 67823 {
        let offset = value - 2288;
        bytes.put_u8(249);
        bytes.put_u8((offset >> 8) as u8);
        bytes.put_u8((offset & 0xFF) as u8);
    } else if value <= 16777215 {
        let offset = value - 67824;
        bytes.put_u8(250);
        bytes.put_u8((offset >> 16) as u8);
        bytes.put_u8((offset >> 8) as u8);
        bytes.put_u8((offset & 0xFF) as u8);
    } else {
        let offset = value - 16777216;
        bytes.put_u8(251);
        bytes.put_u8((offset >> 24) as u8);
        bytes.put_u8((offset >> 16) as u8);
        bytes.put_u8((offset >> 8) as u8);
        bytes.put_u8((offset & 0xFF) as u8);
    }

    bytes
}
