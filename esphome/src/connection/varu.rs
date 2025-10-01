use bytes::{BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt},
    net::TcpStream,
};

pub trait Varu32: AsyncReadExt {
    async fn read_varu32(&mut self) -> io::Result<u32>;
}

impl Varu32 for TcpStream {
    async fn read_varu32(&mut self) -> io::Result<u32> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let mut buf = [0u8; 1];
            self.read_exact(&mut buf).await?;
            let byte = buf[0];

            result |= ((byte & 0x7f) as u32) << shift;
            shift += 7;

            if byte & 0x80 == 0 {
                break;
            }

            if shift >= 32 || (shift == 28 && byte > 0x0F) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Varu32 too long",
                ));
            }
        }

        Ok(result)
    }
}

pub fn varu32_to_bytes(mut value: u32) -> BytesMut {
    let mut bytes = BytesMut::new();

    if value <= 0x7F {
        bytes.put_u8(value as u8);
        return bytes;
    }

    while value != 0 {
        let temp = (value & 0x7F) as u8;
        value >>= 7;
        match value == 0 {
            false => bytes.put_u8(temp | 0x80),
            true => bytes.put_u8(temp),
        }
    }

    bytes
}
