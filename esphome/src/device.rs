use bytes::BytesMut;
use igloo_interface::{Component, ipc::IglooMessage};
use prost::Message;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

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

#[derive(Clone, Debug)]
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
    entity_key_to_index: HashMap<u32, usize>,
    /// maps Igloo entity index -> ESPHome type,key
    entity_index_to_info: Vec<(EntityType, u32)>,
    next_entity_index: usize,
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
    #[error("sending to Igloo write task: `{0}`")]
    IglooSendError(#[from] kanal::SendError),
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
            entity_key_to_index: HashMap::new(),
            entity_index_to_info: Vec::new(),
            next_entity_index: 0,
        }
    }

    pub async fn run(
        mut self,
        igloo_tx: kanal::AsyncSender<IglooMessage>,
        in_rx: kanal::AsyncReceiver<(usize, Vec<Component>)>,
    ) -> Result<(), DeviceError> {
        if !self.connected {
            unreachable!()
        }

        // publish entities
        self.register_entities(&igloo_tx).await?; // TODO timeout, then crash

        self.subscribe_states().await?;

        loop {
            tokio::select! {
                Ok((eidx, comps)) = in_rx.recv() => {
                    self.process_igloo_write(eidx, comps).await?;
                },

                result = self.connection.recv_msg() => {
                    match result {
                        Ok((msg_type, msg)) => {
                            if let Err(e) = self.process_msg(&igloo_tx, msg_type, msg).await {
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

        // TODO fixme drop unexpected messages instead of failing

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

    #[inline]
    async fn process_igloo_write(
        &mut self,
        eindex: usize,
        comps: Vec<Component>,
    ) -> Result<(), DeviceError> {
        let Some((entity_type, key)) = self.entity_index_to_info.get(eindex) else {
            eprintln!(
                "Igloo send update for unknown entity {eindex} on device {}",
                self.id
            );
            return Ok(());
        };

        match entity_type {
            EntityType::Light => entity::light::process(self, *key, comps).await,
            EntityType::Switch => entity::switch::process(self, *key, comps).await,
            EntityType::Button => entity::button::process(self, *key, comps).await,
            EntityType::Number => entity::number::process(self, *key, comps).await,
            EntityType::Select => entity::select::process(self, *key, comps).await,
            EntityType::Text => entity::text::process(self, *key, comps).await,
            EntityType::Fan => entity::fan::process(self, *key, comps).await,
            EntityType::Cover => entity::cover::process(self, *key, comps).await,
            EntityType::Valve => entity::valve::process(self, *key, comps).await,
            EntityType::Siren => entity::siren::process(self, *key, comps).await,
            EntityType::Lock => entity::lock::process(self, *key, comps).await,
            EntityType::MediaPlayer => entity::media_player::process(self, *key, comps).await,
            EntityType::Date => entity::date::process(self, *key, comps).await,
            EntityType::Time => entity::time::process(self, *key, comps).await,
            EntityType::DateTime => entity::date_time::process(self, *key, comps).await,
            EntityType::AlarmControlPanel => {
                entity::alarm_control_panel::process(self, *key, comps).await
            }
            EntityType::Update => entity::update::process(self, *key, comps).await,
            EntityType::Climate => entity::climate::process(self, *key, comps).await,

            _ => {
                eprintln!("{entity_type:#?} currently does not support commands. Skipping..");
                Ok(())
            }
        }
    }

    #[inline]
    async fn process_msg(
        &mut self,
        igloo_tx: &kanal::AsyncSender<IglooMessage>,
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
                self.process_state_update(igloo_tx, msg_type, msg).await?;
            }
        }
        Ok(())
    }

    #[inline]
    pub async fn process_state_update(
        &mut self,
        igloo_tx: &kanal::AsyncSender<IglooMessage>,
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
                self.apply_entity_update(igloo_tx, api::BinarySensorStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::CoverStateResponse => {
                self.apply_entity_update(igloo_tx, api::CoverStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::FanStateResponse => {
                self.apply_entity_update(igloo_tx, api::FanStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::LightStateResponse => {
                self.apply_entity_update(igloo_tx, api::LightStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SensorStateResponse => {
                self.apply_entity_update(igloo_tx, api::SensorStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SwitchStateResponse => {
                self.apply_entity_update(igloo_tx, api::SwitchStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::TextSensorStateResponse => {
                self.apply_entity_update(igloo_tx, api::TextSensorStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::ClimateStateResponse => {
                self.apply_entity_update(igloo_tx, api::ClimateStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::NumberStateResponse => {
                self.apply_entity_update(igloo_tx, api::NumberStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SelectStateResponse => {
                self.apply_entity_update(igloo_tx, api::SelectStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::SirenStateResponse => {
                self.apply_entity_update(igloo_tx, api::SirenStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::LockStateResponse => {
                self.apply_entity_update(igloo_tx, api::LockStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::MediaPlayerStateResponse => {
                self.apply_entity_update(igloo_tx, api::MediaPlayerStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::AlarmControlPanelStateResponse => {
                self.apply_entity_update(
                    igloo_tx,
                    api::AlarmControlPanelStateResponse::decode(msg)?,
                )
                .await?;
            }
            MessageType::TextStateResponse => {
                self.apply_entity_update(igloo_tx, api::TextStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::DateStateResponse => {
                self.apply_entity_update(igloo_tx, api::DateStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::TimeStateResponse => {
                self.apply_entity_update(igloo_tx, api::TimeStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::ValveStateResponse => {
                self.apply_entity_update(igloo_tx, api::ValveStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::DateTimeStateResponse => {
                self.apply_entity_update(igloo_tx, api::DateTimeStateResponse::decode(msg)?)
                    .await?;
            }
            MessageType::UpdateStateResponse => {
                self.apply_entity_update(igloo_tx, api::UpdateStateResponse::decode(msg)?)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn apply_entity_update<T: EntityUpdate>(
        &self,
        igloo_tx: &kanal::AsyncSender<IglooMessage>,
        update: T,
    ) -> Result<(), DeviceError> {
        if update.should_skip() {
            return Ok(());
        }

        let Some(entity) = self.entity_key_to_index.get(&update.key()) else {
            // TODO log err - update for unknown entity
            return Ok(());
        };

        igloo_tx
            .send(IglooMessage::WriteComponents {
                device: self.id,
                entity: *entity,
                comps: update.comps(),
            })
            .await?;

        Ok(())
    }

    pub async fn register_entities(
        &mut self,
        igloo_tx: &kanal::AsyncSender<IglooMessage>,
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
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::BinarySensor,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesCoverResponse => {
                    let msg = api::ListEntitiesCoverResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Cover,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesFanResponse => {
                    let msg = api::ListEntitiesFanResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Fan,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesLightResponse => {
                    let msg = api::ListEntitiesLightResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Light,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesSensorResponse => {
                    let msg = api::ListEntitiesSensorResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Sensor,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesSwitchResponse => {
                    let msg = api::ListEntitiesSwitchResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Switch,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesTextSensorResponse => {
                    let msg = api::ListEntitiesTextSensorResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::TextSensor,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesCameraResponse => {
                    let msg = api::ListEntitiesCameraResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Camera,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesClimateResponse => {
                    let msg = api::ListEntitiesClimateResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Climate,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesNumberResponse => {
                    let msg = api::ListEntitiesNumberResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Number,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesSelectResponse => {
                    let msg = api::ListEntitiesSelectResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Select,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesSirenResponse => {
                    let msg = api::ListEntitiesSirenResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Siren,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesLockResponse => {
                    let msg = api::ListEntitiesLockResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Lock,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesButtonResponse => {
                    let msg = api::ListEntitiesButtonResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Button,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesMediaPlayerResponse => {
                    let msg = api::ListEntitiesMediaPlayerResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::MediaPlayer,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesAlarmControlPanelResponse => {
                    let msg = api::ListEntitiesAlarmControlPanelResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::AlarmControlPanel,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesTextResponse => {
                    let msg = api::ListEntitiesTextResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Text,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesDateResponse => {
                    let msg = api::ListEntitiesDateResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Date,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesTimeResponse => {
                    let msg = api::ListEntitiesTimeResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Time,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesEventResponse => {
                    let msg = api::ListEntitiesEventResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Event,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesValveResponse => {
                    let msg = api::ListEntitiesValveResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Valve,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesDateTimeResponse => {
                    let msg = api::ListEntitiesDateTimeResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::DateTime,
                        msg.comps(),
                    )
                    .await?;
                }
                MessageType::ListEntitiesUpdateResponse => {
                    let msg = api::ListEntitiesUpdateResponse::decode(msg)?;
                    self.register_entity(
                        igloo_tx,
                        msg.name.to_string(),
                        msg.key,
                        EntityType::Update,
                        msg.comps(),
                    )
                    .await?;
                }
                _ => continue,
            }
        }
        Ok(())
    }

    pub async fn register_entity(
        &mut self,
        igloo_tx: &kanal::AsyncSender<IglooMessage>,
        entity_name: String,
        key: u32,
        entity_type: EntityType,
        comps: Vec<Component>,
    ) -> Result<usize, DeviceError> {
        let entity_index = self.next_entity_index;

        igloo_tx
            .send(IglooMessage::RegisterEntity {
                device: self.id,
                entity_name,
                entity_index,
            })
            .await?;

        self.entity_key_to_index.insert(key, self.next_entity_index);
        self.entity_index_to_info.push((entity_type, key));
        self.next_entity_index += 1;

        igloo_tx
            .send(IglooMessage::WriteComponents {
                device: self.id,
                entity: entity_index,
                comps,
            })
            .await?;

        Ok(entity_index)
    }
}
