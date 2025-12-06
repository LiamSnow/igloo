use crate::{Component, ipc::IglooMessage};
use bincode::{config, error::EncodeError};
use rustc_hash::FxHashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::unix::OwnedWriteHalf;

#[derive(Debug)]
pub struct IWriter {
    writer: OwnedWriteHalf,
    scratch: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum IWriterError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Encode Error: {0}")]
    Encode(#[from] EncodeError),
}

impl IWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer,
            scratch: Vec::with_capacity(1024),
        }
    }

    pub async fn write(&mut self, msg: &IglooMessage) -> Result<(), IWriterError> {
        self.scratch.clear();
        bincode::encode_into_std_write(msg, &mut self.scratch, config::standard())?;

        self.writer.write_u32(self.scratch.len() as u32).await?;
        self.writer.write_all(&self.scratch).await?;

        Ok(())
    }

    /// WARN only for Igloo server
    pub fn try_write_immut(
        &self,
        msg: &IglooMessage,
        scratch: &mut Vec<u8>,
    ) -> Result<(), IWriterError> {
        scratch.clear();
        bincode::encode_into_std_write(msg, scratch, config::standard())?;

        let len = scratch.len() as u32;
        let len_bytes = len.to_be_bytes();
        let n = self.writer.try_write(&len_bytes)?;
        if n != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write length prefix",
            )
            .into());
        }

        let n = self.writer.try_write(scratch)?;
        if n != scratch.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write message data",
            )
            .into());
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush().await
    }

    pub fn get_ref(&self) -> &OwnedWriteHalf {
        &self.writer
    }

    pub fn get_mut(&mut self) -> &mut OwnedWriteHalf {
        &mut self.writer
    }

    pub fn scratch_capacity(&self) -> usize {
        self.scratch.capacity()
    }

    pub async fn whats_up_igloo(&mut self, msic: u16, msim: u8) -> Result<(), IWriterError> {
        self.write(&IglooMessage::WhatsUpIgloo { msic, msim }).await
    }

    pub async fn create_device(&mut self, name: String) -> Result<(), IWriterError> {
        self.write(&IglooMessage::CreateDevice(name)).await
    }

    pub async fn device_created(
        &mut self,
        name: String,
        device_id: u64,
    ) -> Result<(), IWriterError> {
        self.write(&IglooMessage::DeviceCreated(name, device_id))
            .await
    }

    pub async fn register_entity(
        &mut self,
        device: u64,
        entity_name: String,
        entity_index: usize,
    ) -> Result<(), IWriterError> {
        self.write(&IglooMessage::RegisterEntity {
            device,
            entity_id: entity_name,
            entity_index,
        })
        .await
    }

    pub async fn write_components(
        &mut self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> Result<(), IWriterError> {
        self.write(&IglooMessage::WriteComponents {
            device,
            entity,
            comps,
        })
        .await
    }

    pub async fn write_custom(
        &mut self,
        name: String,
        params: FxHashMap<String, String>,
    ) -> Result<(), IWriterError> {
        self.write(&IglooMessage::Custom { name, params }).await
    }
}
