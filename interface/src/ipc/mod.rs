use crate::{Component, ipc::codec::LengthDelimitedJSONCodec};
use futures_util::{Sink, SinkExt};
pub use model::*;
use std::{env, io};
use tokio::net::{
    UnixStream,
    unix::{OwnedReadHalf, OwnedWriteHalf},
};
use tokio_util::codec::{FramedRead, FramedWrite};

pub mod codec;
pub mod model;

/// Igloo -> Extension
pub type IWriter = FramedWrite<OwnedWriteHalf, LengthDelimitedJSONCodec<IglooToExtension>>;
/// Igloo -> Extension
pub type IReader = FramedRead<OwnedReadHalf, LengthDelimitedJSONCodec<ExtensionToIgloo>>;

/// Extension -> Igloo
pub type EWriter = FramedWrite<OwnedWriteHalf, LengthDelimitedJSONCodec<ExtensionToIgloo>>;
/// Extension -> Igloo
pub type EReader = FramedRead<OwnedReadHalf, LengthDelimitedJSONCodec<IglooToExtension>>;

pub async fn connect() -> io::Result<(EWriter, EReader)> {
    let stream = UnixStream::connect("igloo.sock").await?;

    let (reader, writer) = stream.into_split();

    let mut writer = FramedWrite::new(writer, LengthDelimitedJSONCodec::new());
    let reader = FramedRead::new(reader, LengthDelimitedJSONCodec::new());

    writer.whats_up_igloo().await?;
    writer.flush().await?;

    Ok((writer, reader))
}

pub fn get_data_path() -> String {
    env::var(DATA_PATH_ENV_VAR).unwrap()
}

pub trait AsyncWriteExtensionToIglooMut {
    type Error;

    fn whats_up_igloo(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn create_device(
        &mut self,
        name: String,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn register_entity(
        &mut self,
        device: u64,
        entity_id: String,
        entity_index: usize,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn write_component(
        &mut self,
        device: u64,
        entity: usize,
        comp: Component,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn write_components(
        &mut self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait AsyncWriteExtensionToIgloo {
    type Error;

    fn whats_up_igloo(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn create_device(&self, name: String) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn register_entity(
        &self,
        device: u64,
        entity_id: String,
        entity_index: usize,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn write_component(
        &self,
        device: u64,
        entity: usize,
        comp: Component,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn write_components(
        &self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<T> AsyncWriteExtensionToIglooMut for T
where
    T: Sink<ExtensionToIgloo, Error = io::Error> + Unpin + Send,
{
    type Error = io::Error;

    async fn whats_up_igloo(&mut self) -> io::Result<()> {
        self.feed(ExtensionToIgloo::WhatsUpIgloo).await
    }

    async fn create_device(&mut self, name: String) -> io::Result<()> {
        self.feed(ExtensionToIgloo::CreateDevice { name }).await
    }

    async fn register_entity(
        &mut self,
        device: u64,
        entity_id: String,
        entity_index: usize,
    ) -> io::Result<()> {
        self.feed(ExtensionToIgloo::RegisterEntity {
            device,
            entity_id,
            entity_index,
        })
        .await
    }

    async fn write_component(
        &mut self,
        device: u64,
        entity: usize,
        comp: Component,
    ) -> io::Result<()> {
        self.write_components(device, entity, vec![comp]).await
    }

    async fn write_components(
        &mut self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> io::Result<()> {
        self.feed(ExtensionToIgloo::WriteComponents {
            device,
            entity,
            comps,
        })
        .await
    }
}

#[cfg(feature = "kanal")]
impl AsyncWriteExtensionToIgloo for kanal::AsyncSender<ExtensionToIgloo> {
    type Error = kanal::SendError;

    async fn whats_up_igloo(&self) -> Result<(), Self::Error> {
        self.send(ExtensionToIgloo::WhatsUpIgloo).await
    }

    async fn create_device(&self, name: String) -> Result<(), Self::Error> {
        self.send(ExtensionToIgloo::CreateDevice { name }).await
    }

    async fn register_entity(
        &self,
        device: u64,
        entity_id: String,
        entity_index: usize,
    ) -> Result<(), Self::Error> {
        self.send(ExtensionToIgloo::RegisterEntity {
            device,
            entity_id,
            entity_index,
        })
        .await
    }

    async fn write_component(
        &self,
        device: u64,
        entity: usize,
        comp: Component,
    ) -> Result<(), Self::Error> {
        self.write_components(device, entity, vec![comp]).await
    }

    async fn write_components(
        &self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> Result<(), Self::Error> {
        self.send(ExtensionToIgloo::WriteComponents {
            device,
            entity,
            comps,
        })
        .await
    }
}

pub trait WriteIglooToExtension {
    fn device_created(
        &mut self,
        name: String,
        device_id: u64,
    ) -> impl Future<Output = io::Result<()>> + Send;

    fn write_component(
        &mut self,
        device: u64,
        entity: usize,
        comp: Component,
    ) -> impl Future<Output = io::Result<()>> + Send;

    fn write_components(
        &mut self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> impl Future<Output = io::Result<()>> + Send;

    fn write_custom(
        &mut self,
        name: String,
        payload: serde_json::Value,
    ) -> impl Future<Output = io::Result<()>> + Send;
}

impl WriteIglooToExtension for IWriter {
    async fn device_created(&mut self, name: String, id: u64) -> io::Result<()> {
        self.feed(IglooToExtension::DeviceCreated { name, id })
            .await
    }

    async fn write_component(
        &mut self,
        device: u64,
        entity: usize,
        comp: Component,
    ) -> io::Result<()> {
        self.write_components(device, entity, vec![comp]).await
    }

    async fn write_components(
        &mut self,
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    ) -> io::Result<()> {
        self.feed(IglooToExtension::WriteComponents {
            device,
            entity,
            comps,
        })
        .await
    }

    async fn write_custom(&mut self, name: String, payload: serde_json::Value) -> io::Result<()> {
        self.feed(IglooToExtension::Custom { name, payload }).await
    }
}
