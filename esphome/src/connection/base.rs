use crate::{connection::error::ConnectionError, model::MessageType};
use bytes::BytesMut;

use super::{noise::NoiseConnection, plain::PlainConnection};

#[allow(async_fn_in_trait)]
pub trait Connectionable {
    async fn send_msg(
        &mut self,
        msg_type: MessageType,
        msg_bytes: &BytesMut,
    ) -> Result<(), ConnectionError>;
    async fn recv_msg(&mut self) -> Result<(MessageType, BytesMut), ConnectionError>;
    async fn connect(&mut self) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
    fn get_name(&self) -> Option<String>;
    async fn readable(&mut self) -> Result<(), ConnectionError>;
}

#[derive(Hash)]
pub enum Connection {
    Noise(NoiseConnection),
    Plain(PlainConnection),
}

impl From<NoiseConnection> for Connection {
    fn from(value: NoiseConnection) -> Self {
        Self::Noise(value)
    }
}

impl From<PlainConnection> for Connection {
    fn from(value: PlainConnection) -> Self {
        Self::Plain(value)
    }
}

impl Connectionable for Connection {
    async fn send_msg(
        &mut self,
        msg_type: MessageType,
        msg_bytes: &BytesMut,
    ) -> Result<(), ConnectionError> {
        match self {
            Connection::Noise(con) => con.send_msg(msg_type, msg_bytes).await,
            Connection::Plain(con) => con.send_msg(msg_type, msg_bytes).await,
        }
    }

    async fn recv_msg(&mut self) -> Result<(MessageType, BytesMut), ConnectionError> {
        match self {
            Connection::Noise(con) => con.recv_msg().await,
            Connection::Plain(con) => con.recv_msg().await,
        }
    }

    async fn connect(&mut self) -> Result<(), ConnectionError> {
        match self {
            Connection::Noise(con) => con.connect().await,
            Connection::Plain(con) => con.connect().await,
        }
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        match self {
            Connection::Noise(con) => con.disconnect().await,
            Connection::Plain(con) => con.disconnect().await,
        }
    }

    fn get_name(&self) -> Option<String> {
        match self {
            Connection::Noise(con) => con.get_name(),
            Connection::Plain(con) => con.get_name(),
        }
    }

    async fn readable(&mut self) -> Result<(), ConnectionError> {
        match self {
            Connection::Noise(con) => con.readable().await,
            Connection::Plain(con) => con.readable().await,
        }
    }
}
