use borsh::{BorshDeserialize, BorshSerialize};
use bytes::BytesMut;
use futures_util::StreamExt;
use igloo_interface::{
    Color, ColorMode, DESELECT_ENTITY, END_TRANSACTION, FloeReaderDefault, FloeWriterDefault,
    SELECT_ENTITY, SelectEntity, StartDeviceTransaction, WRITE_COLOR, WRITE_COLOR_MODE,
    WRITE_COLOR_TEMPERATURE, WRITE_DIMMER, WRITE_SWITCH,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::sync::{Mutex, mpsc};

use crate::{
    api::{self, LightCommandRequest},
    connection::{
        base::{Connection, Connectionable},
        error::ConnectionError,
        noise::NoiseConnection,
        plain::PlainConnection,
    },
    entity::EntityUpdate,
    model::MessageType,
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
            .await
            .unwrap();

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
        shared_reader: Arc<Mutex<FloeReaderDefault>>,
        mut start_trans: mpsc::Receiver<()>,
        end_trans: mpsc::Sender<()>,
    ) -> Result<(), DeviceError> {
        self.shared_writer = Some(shared_writer);

        self.subscribe_states().await?;

        loop {
            // TODO FIXME ok so technically this works fine, but it means
            // that while igloo is communicating a transaction with us
            // we cannot read from the device
            // We _should_ spawn two separate tasks for this, assuming
            // ESPHome can handle that fine
            // Maybe it doesn't matter thought because igloo _may_
            // not be able handle bidrectional communications anyways
            // maybe just keep transactions short idk
            tokio::select! {
                _ = start_trans.recv() => {
                    println!("[Device] Recieved Start Transaction, Waiting for Reader Lock");
                    let mut reader = shared_reader.lock().await;
                    println!("[Device] Got Reader Lock. Sending End Transaction");
                    end_trans.send(()).await.unwrap();
                    self.handle_transaction(&mut reader).await;
                    drop(reader);
                },

                res = self.connection.readable() => if res.is_ok()
                    && let Err(e) = self.recv_process_msg().await // TODO what about other errors??
                        && matches!(e, DeviceError::DeviceRequestShutdown) {
                            return Ok(());
                        }
            }
        }
    }

    async fn handle_transaction(&mut self, reader: &mut FloeReaderDefault) {
        while let Some(res) = reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Frame read error: {e}");
                    continue;
                }
            };

            match cmd_id {
                SELECT_ENTITY => {
                    let res: SelectEntity = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                    let (entity_type, key) = self.entity_idx_to_info.get(&res.entity_idx).unwrap(); // FIXME unwrap
                    match entity_type {
                        EntityType::Light => {
                            self.handle_light_entity_transaction(reader, *key).await;
                        }
                        _ => todo!(),
                    }
                }

                END_TRANSACTION => {
                    return;
                }

                cmd_id => {
                    eprintln!(
                        "Igloo sent unexpected command {cmd_id} during device transaction while no entity was selected"
                    );
                }
            }
        }
    }

    async fn handle_light_entity_transaction(&mut self, reader: &mut FloeReaderDefault, key: u32) {
        let mut req = LightCommandRequest {
            key,
            ..Default::default()
        };

        while let Some(res) = reader.next().await {
            let (cmd_id, payload) = match res {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Frame read error: {e}");
                    continue;
                }
            };

            match cmd_id {
                WRITE_COLOR => {
                    let color: Color = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                    req.has_rgb = true;
                    req.red = (color.r as f32) / 255.;
                    req.green = (color.g as f32) / 255.;
                    req.blue = (color.b as f32) / 255.;
                }

                WRITE_DIMMER => {
                    let val: f32 = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                    req.has_color_brightness = true;
                    req.color_brightness = val;
                    req.has_brightness = true;
                    req.brightness = val;
                }
                WRITE_SWITCH => {
                    let state: bool = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                    req.has_state = true;
                    req.state = state;
                }
                WRITE_COLOR_TEMPERATURE => {
                    let temp: u32 = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                    req.has_color_temperature = true;
                    req.color_temperature = temp as f32;
                }
                WRITE_COLOR_MODE => {
                    let mode: ColorMode = borsh::from_slice(&payload).unwrap(); // FIXME unwrap
                    req.has_color_mode = true;
                    req.color_mode = match mode {
                        ColorMode::RGB => 35,
                        ColorMode::Temperature => 11,
                    };
                }

                DESELECT_ENTITY => {
                    break;
                }

                END_TRANSACTION => {
                    unreachable!(
                        "Igloo tried to end the device transaction without deselecting the entity"
                    );
                }

                // skip other entities
                // TODO maybe log this or..?
                _ => {}
            }
        }

        self.send_msg(MessageType::LightCommandRequest, &req)
            .await
            .unwrap(); // FIXME unwrap
    }

    async fn recv_process_msg(&mut self) -> Result<(), DeviceError> {
        let (msg_type, msg) = self.connection.recv_msg().await?;

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

        let shared_writer = self.shared_writer.as_ref().unwrap();
        let mut writer = shared_writer.lock().await;

        writer
            .start_device_transaction(&StartDeviceTransaction {
                device_idx: self.device_idx.unwrap(),
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

// TODO generate
#[derive(Clone)]
pub enum EntityType {
    BinarySensor,
    Cover,
    Fan,
    Light,
    Sensor,
    Switch,
    TextSensor,
    Camera,
    Climate,
    Number,
    Select,
    Siren,
    Lock,
    Button,
    MediaPlayer,
    AlarmControlPanel,
    Text,
    Date,
    Time,
    Event,
    Valve,
    DateTime,
    Update,
}
