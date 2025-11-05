use borsh::{BorshDeserialize, BorshSerialize};
use bytes::BytesMut;
use igloo_interface::{
    DESELECT_ENTITY, SELECT_ENTITY, SelectEntity, StartTransaction, floe::FloeWriterDefault,
};
use prost::Message;
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
    entity::{self, EntityRegister, EntityUpdate},
    model::{EntityType, MessageType},
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct ConnectionParams {
    pub ip: String,
    pub noise_psk: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
}

pub struct Device {
    pub id: u64,
    pub params: ConnectionParams,
    pub connection: Connection,
    password: String,
    connected: bool,
    last_ping: Option<SystemTime>,
    /// maps ESPHome entity key -> Igloo entity index
    entity_key_to_idx: HashMap<u32, u32>,
    /// maps Igloo entity index -> ESPHome type,key
    entity_idx_to_info: Vec<(EntityType, u32)>,
    next_entity_idx: u32,
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
    pub fn new(id: u64, params: ConnectionParams) -> Self {
        let connection = match &params.noise_psk {
            Some(noise_psk) => {
                NoiseConnection::new(params.ip.clone(), noise_psk.to_string()).into()
            }
            None => PlainConnection::new(params.ip.clone()).into(),
        };

        Device {
            id,
            connection,
            password: params.password.clone().unwrap_or_default(),
            params,
            connected: false,
            last_ping: None,
            entity_key_to_idx: HashMap::new(),
            entity_idx_to_info: Vec::new(),
            next_entity_idx: 0,
        }
    }

    pub async fn run(
        mut self,
        shared_writer: Arc<Mutex<FloeWriterDefault>>,
        mut stream_rx: mpsc::Receiver<(u16, Vec<u8>)>,
    ) -> Result<(), DeviceError> {
        if !self.connected {
            unreachable!()
        }

        // publish entities
        let mut writer = shared_writer.lock().await;
        writer
            .start_transaction(&StartTransaction { device_id: self.id })
            .await?;
        self.register_entities(&mut writer).await?; // TODO timeout, then crash
        writer.end_transaction().await?;
        drop(writer);

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
                            let Some(entity_info) = self.entity_idx_to_info.get(res.entity_idx as usize) else {
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
                            if let Err(e) = self.process_msg(&shared_writer, msg_type, msg).await {
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

    pub async fn connect(&mut self) -> Result<api::DeviceInfoResponse, DeviceError> {
        if self.connected {
            unreachable!();
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
        shared_writer: &Arc<Mutex<FloeWriterDefault>>,
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
                self.process_state_update(shared_writer, msg_type, msg)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn process_state_update(
        &mut self,
        shared_writer: &Arc<Mutex<FloeWriterDefault>>,
        msg_type: MessageType,
        msg: BytesMut,
    ) -> Result<(), DeviceError> {
        match msg_type {
            MessageType::DisconnectRequest
            | MessageType::PingRequest
            | MessageType::PingResponse
            | MessageType::GetTimeRequest
            | MessageType::SubscribeLogsResponse => unreachable!(),
            MessageType::BinarySensorStateResponse => {
                self.apply_entity_update(
                    shared_writer,
                    api::BinarySensorStateResponse::decode(msg)?,
                )
                .await?;
            }
            MessageType::CoverStateResponse => {
                self.apply_entity_update(shared_writer, api::CoverStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::FanStateResponse => {
                self.apply_entity_update(shared_writer, api::FanStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::LightStateResponse => {
                self.apply_entity_update(shared_writer, api::LightStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SensorStateResponse => {
                self.apply_entity_update(shared_writer, api::SensorStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SwitchStateResponse => {
                self.apply_entity_update(shared_writer, api::SwitchStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::TextSensorStateResponse => {
                self.apply_entity_update(shared_writer, api::TextSensorStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::ClimateStateResponse => {
                self.apply_entity_update(shared_writer, api::ClimateStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::NumberStateResponse => {
                self.apply_entity_update(shared_writer, api::NumberStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SelectStateResponse => {
                self.apply_entity_update(shared_writer, api::SelectStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SirenStateResponse => {
                self.apply_entity_update(shared_writer, api::SirenStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::LockStateResponse => {
                self.apply_entity_update(shared_writer, api::LockStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::MediaPlayerStateResponse => {
                self.apply_entity_update(
                    shared_writer,
                    api::MediaPlayerStateResponse::decode(msg)?,
                )
                .await?;
            }
            MessageType::AlarmControlPanelStateResponse => {
                self.apply_entity_update(
                    shared_writer,
                    api::AlarmControlPanelStateResponse::decode(msg)?,
                )
                .await?;
            }
            MessageType::TextStateResponse => {
                self.apply_entity_update(shared_writer, api::TextStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::DateStateResponse => {
                self.apply_entity_update(shared_writer, api::DateStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::TimeStateResponse => {
                self.apply_entity_update(shared_writer, api::TimeStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::ValveStateResponse => {
                self.apply_entity_update(shared_writer, api::ValveStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::DateTimeStateResponse => {
                self.apply_entity_update(shared_writer, api::DateTimeStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::UpdateStateResponse => {
                self.apply_entity_update(shared_writer, api::UpdateStateResponse::decode(msg)?)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn apply_entity_update<T: EntityUpdate>(
        &self,
        shared_writer: &Arc<Mutex<FloeWriterDefault>>,
        update: T,
    ) -> Result<(), DeviceError> {
        if update.should_skip() {
            return Ok(());
        }

        let Some(entity_idx) = self.entity_key_to_idx.get(&update.key()) else {
            // TODO log err - update for unknown entity
            return Ok(());
        };

        let mut writer = shared_writer.lock().await;

        writer
            .start_transaction(&StartTransaction { device_id: self.id })
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

    pub async fn register_entities(
        &mut self,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), DeviceError> {
        self.send_msg(
            MessageType::ListEntitiesRequest,
            &api::ListEntitiesRequest {},
        )
        .await?;
        loop {
            let (msg_type, msg) = self.connection.recv_msg().await?;
            match msg_type {
                MessageType::ListEntitiesServicesResponse => {
                    continue;
                }
                MessageType::ListEntitiesDoneResponse => break,
                MessageType::ListEntitiesBinarySensorResponse => {
                    let msg = api::ListEntitiesBinarySensorResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesCoverResponse => {
                    let msg = api::ListEntitiesCoverResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesFanResponse => {
                    let msg = api::ListEntitiesFanResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesLightResponse => {
                    let msg = api::ListEntitiesLightResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesSensorResponse => {
                    let msg = api::ListEntitiesSensorResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesSwitchResponse => {
                    let msg = api::ListEntitiesSwitchResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesTextSensorResponse => {
                    let msg = api::ListEntitiesTextSensorResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesCameraResponse => {
                    let msg = api::ListEntitiesCameraResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesClimateResponse => {
                    let msg = api::ListEntitiesClimateResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesNumberResponse => {
                    let msg = api::ListEntitiesNumberResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesSelectResponse => {
                    let msg = api::ListEntitiesSelectResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesSirenResponse => {
                    let msg = api::ListEntitiesSirenResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesLockResponse => {
                    let msg = api::ListEntitiesLockResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesButtonResponse => {
                    let msg = api::ListEntitiesButtonResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesMediaPlayerResponse => {
                    let msg = api::ListEntitiesMediaPlayerResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesAlarmControlPanelResponse => {
                    let msg = api::ListEntitiesAlarmControlPanelResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesTextResponse => {
                    let msg = api::ListEntitiesTextResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesDateResponse => {
                    let msg = api::ListEntitiesDateResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesTimeResponse => {
                    let msg = api::ListEntitiesTimeResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesEventResponse => {
                    let msg = api::ListEntitiesEventResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesValveResponse => {
                    let msg = api::ListEntitiesValveResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesDateTimeResponse => {
                    let msg = api::ListEntitiesDateTimeResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                MessageType::ListEntitiesUpdateResponse => {
                    let msg = api::ListEntitiesUpdateResponse::decode(msg)?;
                    msg.register(self, writer).await?;
                }
                _ => continue,
            }
            writer.deselect_entity().await?;
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
        self.entity_idx_to_info.push((entity_type, key));

        writer
            .select_entity(&SelectEntity {
                entity_idx: self.next_entity_idx,
            })
            .await?;

        self.next_entity_idx += 1;

        Ok(())
    }
}
