use bytes::BytesMut;
use std::io;
use tokio_util::codec::Decoder;

pub struct FloeCodec {
    state: DecodeState,
}

impl FloeCodec {
    pub fn new() -> Self {
        FloeCodec {
            state: DecodeState::ReadingLength,
        }
    }
}

#[derive(Debug)]
enum DecodeState {
    ReadingLength,
    ReadingCmdId { total_length: u32 },
    ReadingPayload { cmd_id: u16, payload_length: usize },
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
