use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("not connected")]
    NotConnected,
    #[error("unknown message type `{0}`")]
    UnknownMessageType(u16),
    #[error("noise decrypt error `{0}`")]
    NoiseDecrypt(#[from] snow::error::Error),
    #[error("io error `{0}`")]
    TcpIO(#[from] std::io::Error),
    #[error("base64 decode slice error `{0}` (noise_psk may be incorrectly sized)")]
    Base64DecodeSlice(#[from] base64::DecodeSliceError),
    #[error("client wants unknown noise protocol `{0}`")]
    ClientWantsUnknownNoiseProtocol(u8),
    #[error("recieved message missing null terminator")]
    MessageMissingNullTerminator,
    #[error("handshake had wrong preamble `{0}`")]
    HandshakeHadWrongPreamble(u8),
    #[error("frame had wrong preamble `{0}` (may have wrong Connection type)")]
    FrameHadWrongPreamble(u8),
}
