pub mod codec;
pub use codec::*;
pub mod varu;

use borsh::BorshSerialize;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
};
use tokio_util::codec::FramedRead;

use crate::{MAX_SUPPORTED_COMPONENT, WhatsUpIgloo};

pub const DEFAULT_COMMAND_SIZE: usize = 64;

pub type FloeWriterDefault = FloeWriter<BufWriter<OwnedWriteHalf>>;
pub type FloeReaderDefault = FramedRead<OwnedReadHalf, FloeCodec>;

#[derive(Debug)]
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
}
