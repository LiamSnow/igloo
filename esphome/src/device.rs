use borsh::{BorshDeserialize, BorshSerialize};
use bytes::BytesMut;
use igloo_interface::{
    DESELECT_ENTITY, FloeWriterDefault, SELECT_ENTITY, SelectEntity, StartDeviceTransaction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    mem,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::sync::{Mutex, mpsc};

use crate::{
    api,
    connection::{
        base::{Connection, Connectionable},
        error::ConnectionError,
        noise::NoiseConnection,
        plain::PlainConnection,
    },
    entity::{self, EntityUpdate},
    model::{EntityType, MessageType},
};

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Debug)]
pub struct ConnectionParams {
    pub ip: String,
    pub noise_psk: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
}

pub struct Device {
    pub connection: Connection,
    pub password: String,
    pub connected: bool,
    pub last_ping: Option<SystemTime>,
    /// maps ESPHome entity key -> Igloo entity index
    pub entity_key_to_idx: HashMap<u32, u16>,
    /// maps Igloo entity index -> ESPHome type,key
    pub entity_idx_to_info: HashMap<u16, (EntityType, u32)>,
    pub device_idx: Option<u16>,
    pub next_entity_idx: u16,
    pub shared_writer: Option<Arc<Mutex<FloeWriterDefault>>>,
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("io error `{0}`")]
    IO(#[from] std::io::Error),
    #[error("not connected")]
    NotConnected,
    #[error("device requested shutdown")]
    DeviceRequestShutdown,
    #[error("invalid password")]
    InvalidPassword,
    #[error("connection error `{0}`")]
    ConnectionError(#[from] ConnectionError),
    #[error("frame had wrong preamble `{0}`")]
    FrameHadWrongPreamble(u8),
    #[error("system time error `{0}`")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("system time error `{0}`")]
    SystemTimeIntCastError(#[from] std::num::TryFromIntError),
    #[error("prost decode error `{0}`")]
    ProstDecodeError(#[from] prost::DecodeError),
    #[error("prost encode error `{0}`")]
    ProstEncodeError(#[from] prost::EncodeError),
    #[error("unknown list entities reponse `{0}`")]
    UnknownListEntitiesResponse(MessageType),
    #[error("unknown entity category `{0}`")]
    UnknownEntityCategory(i32),
    #[error("wrong message type `{0}`")]
    WrongMessageType(MessageType),
    #[error("unknown incoming message type `{0}`")]
    UnknownIncomingMessageType(MessageType),
    #[error("unknown log level `{0}`")]
    UnknownLogLevel(i32),
    #[error("entity doesn't exist: `{0}`")]
    InvalidEntity(u16),
}

impl Device {
    pub fn new(params: ConnectionParams) -> Self {
        let connection = match params.noise_psk {
            Some(noise_psk) => NoiseConnection::new(params.ip, noise_psk).into(),
            None => PlainConnection::new(params.ip).into(),
        };

        Device {
            connection,
            password: params.password.unwrap_or_default(),
            connected: false,
            last_ping: None,
            device_idx: None,
            entity_key_to_idx: HashMap::new(),
            entity_idx_to_info: HashMap::new(),
            next_entity_idx: 0,
            shared_writer: None,
        }
    }

    pub async fn connect(&mut self) -> Result<api::DeviceInfoResponse, DeviceError> {
        if self.connected {
            panic!(); // TODO should this reconnect?
        }

        self.connection.connect().await?;

        let _: api::HelloResponse = self
            .trans_msg(
                MessageType::HelloRequest,
                &api::HelloRequest {
                    client_info: "igloo-esphome".to_string(),
                    api_version_major: 1,
                    api_version_minor: 9,
                },
                MessageType::HelloResponse,
            )
            .await?;

        let res = self
            .trans_msg::<api::ConnectResponse>(
                MessageType::ConnectRequest,
                &api::ConnectRequest {
                    password: self.password.clone(),
                },
                MessageType::ConnectResponse,
            )
            .await;

        if let Ok(msg) = res
            && msg.invalid_password
        {
            return Err(DeviceError::InvalidPassword);
        }

        self.connected = true;

        self.device_info().await
    }

    async fn subscribe_states(&mut self) -> Result<(), DeviceError> {
        self.send_msg(
            MessageType::SubscribeStatesRequest,
            &api::SubscribeStatesRequest {},
        )
        .await?;

        Ok(())
    }

    /// Send disconnect request to device, wait for response, then disconnect socket
    pub async fn disconnect(&mut self) -> Result<(), DeviceError> {
        let _: api::DisconnectResponse = self
            .trans_msg(
                MessageType::DisconnectRequest,
                &api::DisconnectRequest {},
                MessageType::DisconnectResponse,
            )
            .await?;
        self.force_disconnect().await
    }

    /// Disconnect socket (without sending disconnect request to device)
    pub async fn force_disconnect(&mut self) -> Result<(), DeviceError> {
        self.connection.disconnect().await?;
        self.connected = false;
        Ok(())
    }

    pub async fn device_info(&mut self) -> Result<api::DeviceInfoResponse, DeviceError> {
        let res: api::DeviceInfoResponse = self
            .trans_msg(
                MessageType::DeviceInfoRequest,
                &api::DeviceInfoRequest {},
                MessageType::DeviceInfoResponse,
            )
            .await?;
        Ok(res)
    }

    pub async fn send_msg(
        &mut self,
        msg_type: MessageType,
        msg: &impl prost::Message,
    ) -> Result<(), DeviceError> {
        let msg_len = msg.encoded_len();
        let mut bytes = BytesMut::with_capacity(msg_len);
        msg.encode(&mut bytes)?;
        bytes.truncate(msg_len);
        self.connection.send_msg(msg_type, &bytes).await?;
        Ok(())
    }

    async fn recv_msg<U: prost::Message + Default>(
        &mut self,
        expected_msg_type: MessageType,
    ) -> Result<U, DeviceError> {
        let (msg_type, mut msg) = self.connection.recv_msg().await?;
        // TODO maybe just skip?
        if msg_type != expected_msg_type {
            return Err(DeviceError::WrongMessageType(msg_type));
        }
        Ok(U::decode(&mut msg)?)
    }

    async fn trans_msg<U: prost::Message + Default>(
        &mut self,
        req_type: MessageType,
        req: &impl prost::Message,
        res_type: MessageType,
    ) -> Result<U, DeviceError> {
        self.send_msg(req_type, req).await?;
        self.recv_msg(res_type).await
    }

    pub async fn run(
        mut self,
        shared_writer: Arc<Mutex<FloeWriterDefault>>,
        mut stream_rx: mpsc::Receiver<(u16, Vec<u8>)>,
    ) -> Result<(), DeviceError> {
        self.shared_writer = Some(shared_writer);

        self.subscribe_states().await?;

        // collect all messages inside each select|deselect
        // entity pairs then process them together
        let mut cur_entity_info = None;
        let mut cur_entity_trans = Vec::with_capacity(100);

        loop {
            tokio::select! {
                Some((cmd_id, payload)) = stream_rx.recv() => {
                    match cmd_id {
                        SELECT_ENTITY => {
                            let res: SelectEntity = borsh::from_slice(&payload).expect("Failed to parse SelectEntity. Crashing");
                            let Some(entity_info) = self.entity_idx_to_info.get(&res.entity_idx) else {
                                eprintln!("Igloo selected invalid entity {}", res.entity_idx);
                                continue;
                            };
                            cur_entity_info = Some(entity_info.clone());
                            cur_entity_trans.clear();
                        }

                        DESELECT_ENTITY => {
                            if cur_entity_info.is_some() {
                                let res = self.process_entity_trans(
                                    cur_entity_info.take().unwrap(), // always succeeds
                                    mem::take(&mut cur_entity_trans)
                                ).await;
                                if let Err(e) = res {
                                    eprintln!("Error processing entity transaction: {e}");
                                }
                            } else {
                                eprintln!("Igloo deselected nothing");
                            }
                        }

                        cmd_id => {
                            if cur_entity_info.is_some() {
                                cur_entity_trans.push((cmd_id, payload));
                            } else {
                                eprintln!("Device got unexpected command {cmd_id} during transaction.");
                            }
                        }
                    }
                },

                result = self.connection.recv_msg() => {
                    match result {
                        Ok((msg_type, msg)) => {
                            if let Err(e) = self.process_msg(msg_type, msg).await {
                                eprintln!("[Device] Error processing message: {:?}", e);
                                if matches!(e, DeviceError::DeviceRequestShutdown) {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[Device] Error receiving message: {:?}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_entity_trans(
        &mut self,
        info: (EntityType, u32),
        commands: Vec<(u16, Vec<u8>)>,
    ) -> Result<(), DeviceError> {
        let (entity_type, key) = info;
        match entity_type {
            EntityType::Light => entity::light::process(self, key, commands).await,
            EntityType::Switch => entity::switch::process(self, key, commands).await,
            EntityType::Button => entity::button::process(self, key, commands).await,
            EntityType::Number => entity::number::process(self, key, commands).await,
            EntityType::Select => entity::select::process(self, key, commands).await,
            EntityType::Text => entity::text::process(self, key, commands).await,
            EntityType::Fan => entity::fan::process(self, key, commands).await,
            EntityType::Cover => entity::cover::process(self, key, commands).await,
            EntityType::Valve => entity::valve::process(self, key, commands).await,
            EntityType::Siren => entity::siren::process(self, key, commands).await,
            EntityType::Lock => entity::lock::process(self, key, commands).await,
            EntityType::MediaPlayer => entity::media_player::process(self, key, commands).await,
            EntityType::Date => entity::date::process(self, key, commands).await,
            EntityType::Time => entity::time::process(self, key, commands).await,
            EntityType::DateTime => entity::date_time::process(self, key, commands).await,
            EntityType::AlarmControlPanel => {
                entity::alarm_control_panel::process(self, key, commands).await
            }
            EntityType::Update => entity::update::process(self, key, commands).await,
            EntityType::Climate => entity::climate::process(self, key, commands).await,

            _ => {
                eprintln!("{entity_type:#?} currently does not support commands. Skipping..");
                Ok(())
            }
        }
    }

    async fn process_msg(
        &mut self,
        msg_type: MessageType,
        msg: BytesMut,
    ) -> Result<(), DeviceError> {
        match msg_type {
            MessageType::DisconnectRequest => {
                self.send_msg(MessageType::DisconnectResponse, &api::DisconnectResponse {})
                    .await?;
                self.connection.disconnect().await?;
                return Err(DeviceError::DeviceRequestShutdown);
            }
            MessageType::PingRequest => {
                self.send_msg(MessageType::PingResponse, &api::PingResponse {})
                    .await?;
            }
            MessageType::PingResponse => {
                self.last_ping = Some(SystemTime::now());
            }
            MessageType::GetTimeRequest => {
                self.send_msg(
                    MessageType::GetTimeResponse,
                    &api::GetTimeResponse {
                        epoch_seconds: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(DeviceError::SystemTimeError)?
                            .as_secs()
                            .try_into()
                            .map_err(DeviceError::SystemTimeIntCastError)?,
                    },
                )
                .await?;
            }
            MessageType::SubscribeLogsResponse => {
                // TODO how should logs work?
                // Maybe we have a Bool Component "logs_enabled" (default false)
                // for this device and it starts collecting logs to file?
                // Maybe it collects in ram, then has a custom
            }

            _ => {
                if self.shared_writer.is_some() {
                    self.process_state_update(msg_type, msg).await?;
                }
                // TODO else log?
            }
        }
        Ok(())
    }

    pub async fn register_entity(
        &mut self,
        writer: &mut FloeWriterDefault,
        name: &str,
        key: u32,
        entity_type: EntityType,
    ) -> Result<(), std::io::Error> {
        writer
            .register_entity(&igloo_interface::RegisterEntity {
                entity_name: name.to_string(),
                entity_idx: self.next_entity_idx,
            })
            .await?;

        self.entity_key_to_idx.insert(key, self.next_entity_idx);
        self.entity_idx_to_info
            .insert(self.next_entity_idx, (entity_type, key));

        writer
            .select_entity(&SelectEntity {
                entity_idx: self.next_entity_idx,
            })
            .await?;

        self.next_entity_idx += 1;

        Ok(())
    }

    pub async fn apply_entity_update<T: EntityUpdate>(&self, update: T) -> Result<(), DeviceError> {
        if update.should_skip() {
            return Ok(());
        }

        let Some(entity_idx) = self.entity_key_to_idx.get(&update.key()) else {
            // TODO log err - update for unknown entity
            return Ok(());
        };

        let shared_writer = self.shared_writer.as_ref().unwrap(); // always succeeds
        let mut writer = shared_writer.lock().await;

        writer
            .start_device_transaction(&StartDeviceTransaction {
                device_idx: self.device_idx.unwrap(), // always succeeds
            })
            .await?;

        writer
            .select_entity(&SelectEntity {
                entity_idx: *entity_idx,
            })
            .await?;

        update.write_to(&mut writer).await?;

        writer.end_transaction().await?;
        writer.flush().await?;

        Ok(())
    }
}
