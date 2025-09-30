use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{
    CustomCommandErrorPayload, DeviceRegisteredPayload, ExecuteCustomCommandPayload, FloeCommand,
    HandshakeRequestPayload, HandshakeResponsePayload, IglooCommand, LogPayload,
    RegisterDevicePayload, RequestUpdatesPayload, UpdatesPayload,
    codec::{CodecError, FloeCodec},
};

#[derive(Error, Debug)]
pub enum FloeManagerError {
    #[error("Codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("Channel send error")]
    ChannelSend(String),
    #[error("Handshake not received")]
    HandshakeNotReceived,
}

#[trait_variant::make(FloeHandler: Send)]
pub trait LocalFloeHandler {
    async fn on_handshake(&mut self, payload: HandshakeRequestPayload, manager: &FloeManager);

    async fn on_device_registered(
        &mut self,
        payload: DeviceRegisteredPayload,
        manager: &FloeManager,
    );

    async fn on_request_updates(&mut self, payload: RequestUpdatesPayload, manager: &FloeManager);

    async fn on_execute_custom_command(
        &mut self,
        payload: ExecuteCustomCommandPayload,
        manager: &FloeManager,
    );
}

#[derive(Clone)]
pub struct FloeManager {
    tx: mpsc::Sender<FloeCommand>,
}

impl FloeManager {
    pub async fn run<H: FloeHandler>(mut handler: H) -> Result<(), FloeManagerError> {
        let (tx, mut rx) = mpsc::channel::<FloeCommand>(100);
        let manager = FloeManager { tx };

        let writer_handle = tokio::spawn(async move {
            let stdout = tokio::io::stdout();
            let mut framed = FramedWrite::new(stdout, FloeCodec::new());

            while let Some(cmd) = rx.recv().await {
                framed.send(cmd).await?;
            }

            framed.flush().await?;
            Ok::<(), FloeManagerError>(())
        });

        let stdin = tokio::io::stdin();
        let mut framed = FramedRead::new(stdin, FloeCodec::new());

        // wait for handshake
        let handshake_payload = loop {
            match framed.next().await {
                Some(Ok(IglooCommand::HandshakeRequest(payload))) => break payload,
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(e.into()),
                None => return Err(FloeManagerError::HandshakeNotReceived),
            }
        };

        handler.on_handshake(handshake_payload, &manager).await;

        while let Some(result) = framed.next().await {
            let cmd = match result {
                Ok(cmd) => cmd,
                Err(e) => {
                    let _ = manager
                        .log(format!("Failed to decode command: {}", e))
                        .await;
                    continue;
                }
            };

            match cmd {
                IglooCommand::HandshakeRequest(_) => {
                    manager
                        .log("Received unexpected handshake request".to_string())
                        .await?;
                }
                IglooCommand::DeviceRegistered(payload) => {
                    handler.on_device_registered(payload, &manager).await
                }
                IglooCommand::RequestUpdates(payload) => {
                    handler.on_request_updates(payload, &manager).await
                }
                IglooCommand::ExecuteCustomCommand(payload) => {
                    handler.on_execute_custom_command(payload, &manager).await
                }
            }
        }

        drop(manager);
        let _ = writer_handle.await;
        Ok(())
    }

    pub async fn handshake_response(
        &self,
        payload: HandshakeResponsePayload,
    ) -> Result<(), FloeManagerError> {
        self.send_command(FloeCommand::HandshakeResponse(payload))
            .await
    }

    pub async fn register_device(
        &self,
        payload: RegisterDevicePayload,
    ) -> Result<(), FloeManagerError> {
        self.send_command(FloeCommand::RegisterDevice(payload))
            .await
    }

    pub async fn updates(&self, payload: UpdatesPayload) -> Result<(), FloeManagerError> {
        self.send_command(FloeCommand::Updates(payload)).await
    }

    pub async fn custom_command_error(
        &self,
        payload: CustomCommandErrorPayload,
    ) -> Result<(), FloeManagerError> {
        self.send_command(FloeCommand::CustomCommandError(payload))
            .await
    }

    pub async fn log(&self, payload: String) -> Result<(), FloeManagerError> {
        self.send_command(FloeCommand::Log(LogPayload { message: payload }))
            .await
    }

    async fn send_command(&self, cmd: FloeCommand) -> Result<(), FloeManagerError> {
        self.tx
            .send(cmd)
            .await
            .map_err(|_| FloeManagerError::ChannelSend("Failed to send command".to_string()))
    }
}
