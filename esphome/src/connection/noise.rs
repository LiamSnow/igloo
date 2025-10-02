use super::base::Connectionable;
use crate::{connection::error::ConnectionError, model::MessageType};
use base64::prelude::*;
use bytes::{Buf, BytesMut};
use memchr::memchr;
use snow::{HandshakeState, TransportState};
use std::{
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub const NOISE_HELLO: &[u8; 3] = b"\x01\x00\x00";
pub const READ_TIMEOUT: Option<Duration> = Some(Duration::from_secs(60));
pub const NOISE_PARAMS: &str = "Noise_NNpsk0_25519_ChaChaPoly_SHA256";
pub const NOISE_PROLOGUE: &[u8; 14] = b"NoiseAPIInit\x00\x00";
pub const NOISE_PSK_LEN: usize = 32;

pub struct NoiseConnection {
    pub(crate) ip: String,
    noise_psk: String,
    pub(crate) stream: Option<TcpStream>,
    noise: Option<TransportState>,
    pub server_name: Option<String>,
}

impl Hash for NoiseConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
    }
}

impl Connectionable for NoiseConnection {
    async fn send_msg(
        &mut self,
        msg_type: MessageType,
        msg_bytes: &BytesMut,
    ) -> Result<(), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;
        let noise = self.noise.as_mut().ok_or(ConnectionError::NotConnected)?;

        //make frame
        let msg_type = msg_type as usize;
        let msg_len = msg_bytes.len();
        let frame_header = [
            (msg_type >> 8) as u8,
            msg_type as u8,
            (msg_len >> 8) as u8,
            msg_len as u8,
        ];
        //TODO reuse buffer?
        let mut frame = BytesMut::with_capacity(frame_header.len() + msg_len);
        frame.extend_from_slice(&frame_header);
        frame.extend_from_slice(msg_bytes);

        //encrypt frame
        let mut eframe = BytesMut::with_capacity(65535);
        eframe.resize(65535, 0);
        let eframe_len = noise.write_message(&frame, &mut eframe)?;
        eframe.truncate(eframe_len);

        //make packet
        let packet_header = [0x01, (eframe_len >> 8) as u8, eframe_len as u8];
        //TODO reuse buffer?
        let mut packet = BytesMut::with_capacity(packet_header.len() + eframe.len());
        packet.extend_from_slice(&packet_header);
        packet.extend_from_slice(&eframe);

        //send packet
        stream.write_all(&packet).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn recv_msg(&mut self) -> Result<(MessageType, BytesMut), ConnectionError> {
        let stream = self.stream.as_mut().ok_or(ConnectionError::NotConnected)?;
        let noise = self.noise.as_mut().ok_or(ConnectionError::NotConnected)?;
        let frame = Self::read_frame(stream).await?;
        let mut msg = BytesMut::with_capacity(65535);
        msg.resize(65535, 0);
        let msg_size = noise.read_message(&frame, &mut msg)?;
        msg.truncate(msg_size);
        let msg_type_num = u16::from_be_bytes([msg[0], msg[1]]);
        let msg_type = MessageType::from_repr(msg_type_num)
            .ok_or(ConnectionError::UnknownMessageType(msg_type_num))?;
        msg.advance(4);
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
        let mut noise_handshake = Self::setup_noise(&self.noise_psk)?;
        let mut stream = TcpStream::connect(&self.ip).await?;
        Self::send_hello(&mut stream, &mut noise_handshake).await?;
        self.server_name = Some(Self::receive_hello(&mut stream).await?);
        self.noise = Some(Self::receive_handshake(&mut stream, noise_handshake).await?);
        self.stream = Some(stream);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        if let Some(stream) = &mut self.stream {
            stream.shutdown().await?;
        }
        self.noise = None;
        self.stream = None;
        self.server_name = None;
        Ok(())
    }

    fn get_name(&self) -> Option<String> {
        self.server_name.clone()
    }
}

impl NoiseConnection {
    pub fn new(ip: String, noise_psk: String) -> Self {
        Self {
            ip,
            noise_psk,
            stream: None,
            noise: None,
            server_name: None,
        }
    }

    fn setup_noise(noise_psk: &str) -> Result<HandshakeState, ConnectionError> {
        let mut key = [0u8; 32];
        BASE64_STANDARD.decode_slice(noise_psk, &mut key)?;
        Ok(snow::Builder::new(NOISE_PARAMS.parse()?)
            .psk(0, &key)
            .prologue(NOISE_PROLOGUE)
            .build_initiator()?)
    }

    /// Send ClientHello to the server
    async fn send_hello(
        stream: &mut TcpStream,
        noise_handshake: &mut HandshakeState,
    ) -> Result<(), ConnectionError> {
        let mut frame = BytesMut::with_capacity(65535);
        frame.resize(65535, 0);
        let mut frame_len = noise_handshake.write_message(&[], &mut frame)?;
        frame.truncate(frame_len);
        frame_len += 1;
        let header = [0x01, (frame_len >> 8) as u8, frame_len as u8];
        //TODO reuse buffer?
        let mut message = BytesMut::with_capacity(3 + 3 + 1 + frame_len);
        message.extend_from_slice(NOISE_HELLO);
        message.extend_from_slice(&header);
        message.extend_from_slice(&[0x00]);
        message.extend_from_slice(&frame);
        stream.write_all(&message).await?;
        Ok(())
    }

    async fn receive_hello(stream: &mut TcpStream) -> Result<String, ConnectionError> {
        let frame = Self::read_frame(stream).await?;
        if frame[0] != 0x01 {
            return Err(ConnectionError::ClientWantsUnknownNoiseProtocol(frame[0]));
        }
        let pos = memchr(0, &frame[1..]).ok_or(ConnectionError::MessageMissingNullTerminator)?;
        let server_name = String::from_utf8_lossy(&frame[1..pos + 1]).into_owned();
        Ok(server_name)
    }

    async fn receive_handshake(
        stream: &mut TcpStream,
        mut noise_handshake: HandshakeState,
    ) -> Result<TransportState, ConnectionError> {
        let frame = Self::read_frame(stream).await?;
        if frame[0] != 0x00 {
            return Err(ConnectionError::HandshakeHadWrongPreamble(frame[0]));
        }
        noise_handshake.read_message(&frame[1..], &mut [])?;
        Ok(noise_handshake.into_transport_mode()?)
    }

    async fn read_frame(stream: &mut TcpStream) -> Result<BytesMut, ConnectionError> {
        let mut header = [0u8; 3];
        stream.read_exact(&mut header).await?;
        if header[0] != 0x01 {
            return Err(ConnectionError::FrameHadWrongPreamble(header[0]));
        }
        let frame_size = u16::from_be_bytes([header[1], header[2]]) as usize;

        let mut frame = BytesMut::with_capacity(frame_size);
        let size = stream.read_buf(&mut frame).await?;
        frame.truncate(size);
        Ok(frame)
    }
}
