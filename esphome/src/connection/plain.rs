use super::base::Connectionable;
use super::varu::{Varu32, varu32_to_bytes};
use crate::connection::error::ConnectionError;
use crate::model::MessageType;
use bytes::{BufMut, BytesMut};
use std::hash::{Hash, Hasher};
use tokio::io::AsyncReadExt;
use tokio::{io::AsyncWriteExt, net::TcpStream};

///NOTE UNTESTED!!!!!!!!
pub struct PlainConnection {
    pub(crate) ip: String,
    pub(crate) stream: Option<TcpStream>,
}

impl Hash for PlainConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
    }
}

impl Connectionable for PlainConnection {
    async fn send_msg(
        &mut self,
        msg_type: MessageType,
        msg_bytes: &BytesMut,
    ) -> Result<(), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;

        let msg_type_var = varu32_to_bytes(msg_type as u32);
        let msg_len = msg_bytes.len();
        let msg_len_var = varu32_to_bytes(msg_len as u32);

        let mut packet =
            BytesMut::with_capacity(msg_len + 1 + msg_type_var.len() + msg_len_var.len());
        packet.put_u8(0);
        packet.extend_from_slice(&msg_type_var);
        packet.extend_from_slice(&msg_len_var);
        packet.extend_from_slice(&msg_bytes);

        stream.write_all(&packet).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn recv_msg(&mut self) -> Result<(MessageType, BytesMut), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;
        let preamble = stream.read_varu32().await?;
        if preamble != 0x00 {
            return Err(ConnectionError::FrameHadWrongPreamble(preamble as u8));
        }

        let msg_len = stream.read_varu32().await? as usize;
        let msg_type_num = stream.read_varu32().await? as u16;
        let msg_type = MessageType::from_repr(msg_type_num)
            .ok_or(ConnectionError::UnknownMessageType(msg_type_num))?;
        let mut msg = BytesMut::with_capacity(msg_len);
        stream.read_buf(&mut msg).await?;
        Ok((msg_type, msg))
    }

    async fn readable(&mut self) -> Result<(), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;
        stream.readable().await?;
        Ok(())
    }

    async fn connect(&mut self) -> Result<(), ConnectionError> {
        if self.stream.is_some() {
            return Ok(()); //TODO: is this wanted behavior... should this error? should it reconnect?
        }
        let stream = TcpStream::connect(&self.ip).await?;
        self.stream = Some(stream);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        if let Some(stream) = &mut self.stream {
            stream.shutdown().await?;
        }
        self.stream = None;
        Ok(())
    }

    fn get_name(&self) -> Option<String> {
        None
    }
}

impl PlainConnection {
    pub fn new(ip: String) -> Self {
        Self { ip, stream: None }
    }
}
