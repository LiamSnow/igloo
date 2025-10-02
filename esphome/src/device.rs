use async_trait::async_trait;
use borsh::{BorshDeserialize, BorshSerialize};
use bytes::BytesMut;
use futures_util::StreamExt;
use igloo_interface::{
    AlarmState, ClimateMode, Color, ColorMode, CoverState, DESELECT_ENTITY, Date, END_TRANSACTION,
    FanDirection, FanOscillation, FanSpeed, FloeReaderDefault, FloeWriterDefault, LockState,
    MediaState, NumberMode, SELECT_ENTITY, SelectEntity, SensorStateClass, StartDeviceTransaction,
    TextMode, Time, Unit, ValveState, WRITE_COLOR, WRITE_COLOR_MODE, WRITE_COLOR_TEMPERATURE,
    WRITE_DIMMER, WRITE_SWITCH,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::sync::{Mutex, mpsc};

use crate::{
    api::{self, EntityCategory, LightCommandRequest},
    connection::{
        base::{Connection, Connectionable},
        error::ConnectionError,
        noise::NoiseConnection,
        plain::PlainConnection,
    },
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

    async fn send_msg(
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
        mut start_recv: mpsc::Receiver<()>,
    ) -> Result<(), DeviceError> {
        self.shared_writer = Some(shared_writer);

        self.subscribe_states().await?;

        loop {
            tokio::select! {
                res = start_recv.recv() => {
                    if res.is_none() {
                        // our comms to the main loop has been dropped -> shutdown
                        // TODO send disconnect request?
                        return Ok(());
                    }

                    self.handle_transaction(&shared_reader).await;
                },

                res = self.connection.readable() => if res.is_ok()
                    && let Err(e) = self.recv_process_msg().await // TODO what about other errors??
                        && matches!(e, DeviceError::DeviceRequestShutdown) {
                            return Ok(());
                        }
            }
        }
    }

    async fn handle_transaction(&mut self, shared_reader: &Arc<Mutex<FloeReaderDefault>>) {
        let mut reader = shared_reader.lock().await;

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
                            self.handle_light_entity_transaction(&mut reader, *key)
                                .await;
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

    pub async fn register_entities(
        &mut self,
        writer: &mut FloeWriterDefault,
        device_idx: u16,
    ) -> Result<(), DeviceError> {
        self.send_msg(
            MessageType::ListEntitiesRequest,
            &api::ListEntitiesRequest {},
        )
        .await?;

        self.device_idx = Some(device_idx);

        loop {
            let (msg_type, msg) = self.connection.recv_msg().await?;
            use EntityType::*;

            match msg_type {
                MessageType::ListEntitiesServicesResponse => {
                    // TODO should we support services?
                    continue;
                }
                MessageType::ListEntitiesDoneResponse => break,

                MessageType::ListEntitiesBinarySensorResponse => {
                    let msg = api::ListEntitiesBinarySensorResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, BinarySensor)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    writer.sensor().await?;
                }

                MessageType::ListEntitiesCoverResponse => {
                    let msg = api::ListEntitiesCoverResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Cover)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    writer.sensor().await?;
                }

                MessageType::ListEntitiesFanResponse => {
                    let msg = api::ListEntitiesFanResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Fan)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                }

                MessageType::ListEntitiesLightResponse => {
                    let msg = api::ListEntitiesLightResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Light)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                }

                MessageType::ListEntitiesSensorResponse => {
                    let msg = api::ListEntitiesSensorResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Sensor)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_sensor_state_class(writer, msg.state_class()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    add_unit(writer, msg.unit_of_measurement).await?;
                    writer.sensor().await?;
                    writer.accuracy_decimals(msg.accuracy_decimals).await?;
                }

                MessageType::ListEntitiesSwitchResponse => {
                    let msg = api::ListEntitiesSwitchResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Switch)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                }

                MessageType::ListEntitiesTextSensorResponse => {
                    let msg = api::ListEntitiesTextSensorResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, TextSensor)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    writer.sensor().await?;
                }

                MessageType::ListEntitiesCameraResponse => {
                    let msg = api::ListEntitiesCameraResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Camera)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                }

                // The ESPHome climate entity doesn't really match this ECS model
                // So we are breaking it up into a few entities
                MessageType::ListEntitiesClimateResponse => {
                    let msg = api::ListEntitiesClimateResponse::decode(msg)?;

                    // Humidity
                    if msg.supports_current_humidity || msg.supports_target_humidity {
                        let name = format!("{}_humidity", msg.name);
                        self.register_entity(writer, &name, msg.key, Climate)
                            .await?;
                        add_entity_category(writer, msg.entity_category()).await?;
                        add_icon(writer, &msg.icon).await?;
                        add_f32_bounds(
                            writer,
                            msg.visual_min_humidity,
                            msg.visual_max_humidity,
                            None,
                        )
                        .await?;
                        writer.sensor().await?;
                        writer.deselect_entity().await?;
                    }

                    // Current Temperature
                    if msg.supports_current_temperature {
                        let name = format!("{}_current_temperature", msg.name);
                        self.register_entity(writer, &name, msg.key, Climate)
                            .await?;
                        add_entity_category(writer, msg.entity_category()).await?;
                        add_icon(writer, &msg.icon).await?;
                        add_f32_bounds(
                            writer,
                            msg.visual_min_temperature,
                            msg.visual_max_temperature,
                            Some(msg.visual_current_temperature_step),
                        )
                        .await?;
                        writer.sensor().await?;
                        writer.deselect_entity().await?;
                    }

                    // Two Point Temperature
                    if msg.supports_two_point_target_temperature {
                        // TODO verify this is right
                        // Lower
                        {
                            let name = format!("{}_target_lower_temperature", msg.name);
                            self.register_entity(writer, &name, msg.key, Climate)
                                .await?;
                            add_entity_category(writer, msg.entity_category()).await?;
                            add_icon(writer, &msg.icon).await?;
                            add_f32_bounds(
                                writer,
                                msg.visual_min_temperature,
                                msg.visual_max_temperature,
                                Some(msg.visual_target_temperature_step),
                            )
                            .await?;
                            writer.deselect_entity().await?;
                        }

                        // Upper
                        {
                            let name = format!("{}_target_upper_temperature", msg.name);
                            self.register_entity(writer, &name, msg.key, Climate)
                                .await?;
                            add_entity_category(writer, msg.entity_category()).await?;
                            add_icon(writer, &msg.icon).await?;
                            add_f32_bounds(
                                writer,
                                msg.visual_min_temperature,
                                msg.visual_max_temperature,
                                Some(msg.visual_target_temperature_step),
                            )
                            .await?;
                            writer.deselect_entity().await?;
                        }
                    }
                    // One Point Temperature Target
                    else {
                        let name = format!("{}_target_temperature", msg.name);
                        self.register_entity(writer, &name, msg.key, Climate)
                            .await?;
                        add_entity_category(writer, msg.entity_category()).await?;
                        add_icon(writer, &msg.icon).await?;
                        add_f32_bounds(
                            writer,
                            msg.visual_min_temperature,
                            msg.visual_max_temperature,
                            Some(msg.visual_target_temperature_step),
                        )
                        .await?;
                        writer.deselect_entity().await?;
                    }

                    // Climate Mode
                    {
                        let name = format!("{}_mode", msg.name);
                        self.register_entity(writer, &name, msg.key, Climate)
                            .await?;
                        add_entity_category(writer, msg.entity_category()).await?;
                        add_climate_modes(writer, msg.supported_modes()).await?;
                        add_icon(writer, &msg.icon).await?;
                        writer.deselect_entity().await?;
                    }

                    // Fan
                    {
                        let name = format!("{}_fan", msg.name);
                        self.register_entity(writer, &name, msg.key, Climate)
                            .await?;
                        add_entity_category(writer, msg.entity_category()).await?;
                        add_fan_speeds(writer, msg.supported_fan_modes()).await?;
                        add_fan_oscillations(writer, msg.supported_swing_modes()).await?;
                        add_icon(writer, &msg.icon).await?;
                        writer.deselect_entity().await?;
                    }

                    // Preset
                    {
                        let name = format!("{}_preset", msg.name);
                        self.register_entity(writer, &name, msg.key, Climate)
                            .await?;
                        add_entity_category(writer, msg.entity_category()).await?;
                        add_icon(writer, &msg.icon).await?;
                        writer.text_select().await?;
                        writer
                            .text_list(
                                msg.supported_presets()
                                    .map(|preset| format!("{preset:#?}"))
                                    .chain(msg.supported_custom_presets.iter().cloned())
                                    .collect(),
                            )
                            .await?;
                    }
                }

                MessageType::ListEntitiesNumberResponse => {
                    let msg = api::ListEntitiesNumberResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Number)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_number_mode(writer, msg.mode()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    add_f32_bounds(writer, msg.min_value, msg.max_value, Some(msg.step)).await?;
                    add_unit(writer, msg.unit_of_measurement).await?;
                }

                MessageType::ListEntitiesSelectResponse => {
                    let msg = api::ListEntitiesSelectResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Select)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    writer.text_select().await?;
                    writer.text_list(msg.options).await?;
                }

                MessageType::ListEntitiesSirenResponse => {
                    let msg = api::ListEntitiesSirenResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Siren)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    writer.text_select().await?;
                    writer.text_list(msg.tones).await?;
                    writer.siren().await?;
                }

                MessageType::ListEntitiesLockResponse => {
                    let msg = api::ListEntitiesLockResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Lock)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    writer.text(msg.code_format).await?; // TODO is this right?
                }

                MessageType::ListEntitiesButtonResponse => {
                    let msg = api::ListEntitiesButtonResponse::decode(msg)?;
                    // TODO should this have a Button component or something?
                    // to produce an event?
                    self.register_entity(writer, &msg.name, msg.key, Button)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                }

                MessageType::ListEntitiesMediaPlayerResponse => {
                    let msg = api::ListEntitiesMediaPlayerResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, MediaPlayer)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    // TODO .supported_formats
                }

                MessageType::ListEntitiesAlarmControlPanelResponse => {
                    let msg = api::ListEntitiesAlarmControlPanelResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, AlarmControlPanel)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    // TODO .supported_features
                }

                MessageType::ListEntitiesTextResponse => {
                    let msg = api::ListEntitiesTextResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Text)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    writer.text_mode(msg.mode().as_igloo()).await?;
                    writer.text_min_length(msg.min_length).await?;
                    writer.text_max_length(msg.max_length).await?;
                    writer.text_pattern(msg.pattern).await?;
                }

                MessageType::ListEntitiesDateResponse => {
                    let msg = api::ListEntitiesDateResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Date)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                }

                MessageType::ListEntitiesTimeResponse => {
                    let msg = api::ListEntitiesTimeResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Time)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                }

                MessageType::ListEntitiesEventResponse => {
                    let msg = api::ListEntitiesEventResponse::decode(msg)?;
                    // TODO should this have an event component?
                    self.register_entity(writer, &msg.name, msg.key, Event)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    writer.text_list(msg.event_types).await?;
                }

                MessageType::ListEntitiesValveResponse => {
                    let msg = api::ListEntitiesValveResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Valve)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                    writer.valve().await?;
                    // TODO supports position/stop?
                }

                MessageType::ListEntitiesDateTimeResponse => {
                    let msg = api::ListEntitiesDateTimeResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, DateTime)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                }

                MessageType::ListEntitiesUpdateResponse => {
                    let msg = api::ListEntitiesUpdateResponse::decode(msg)?;
                    self.register_entity(writer, &msg.name, msg.key, Update)
                        .await?;
                    add_entity_category(writer, msg.entity_category()).await?;
                    add_icon(writer, &msg.icon).await?;
                    add_device_class(writer, msg.device_class).await?;
                }

                _ => continue,
            }

            writer.deselect_entity().await?;
        }

        Ok(())
    }

    async fn register_entity(
        &mut self,
        writer: &mut FloeWriterDefault,
        name: &str,
        key: u32,
        entity_type: EntityType,
    ) -> Result<(), std::io::Error> {
        writer
            .register_entity(igloo_interface::RegisterEntity {
                entity_name: name.to_string(),
                entity_idx: self.next_entity_idx,
            })
            .await?;

        self.entity_key_to_idx.insert(key, self.next_entity_idx);
        self.entity_idx_to_info
            .insert(self.next_entity_idx, (entity_type, key));

        writer
            .select_entity(SelectEntity {
                entity_idx: self.next_entity_idx,
            })
            .await?;

        self.next_entity_idx += 1;

        Ok(())
    }

    async fn process_state_update(
        &mut self,
        msg_type: MessageType,
        msg: BytesMut,
    ) -> Result<(), DeviceError> {
        match msg_type {
            MessageType::DisconnectRequest
            | MessageType::PingRequest
            | MessageType::PingResponse
            | MessageType::GetTimeRequest
            | MessageType::SubscribeLogsResponse => {
                unreachable!()
            }

            MessageType::BinarySensorStateResponse => {
                self.apply_entity_update(api::BinarySensorStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::CoverStateResponse => {
                self.apply_entity_update(api::CoverStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::FanStateResponse => {
                self.apply_entity_update(api::FanStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::LightStateResponse => {
                self.apply_entity_update(api::LightStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::SensorStateResponse => {
                self.apply_entity_update(api::SensorStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::SwitchStateResponse => {
                self.apply_entity_update(api::SwitchStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::TextSensorStateResponse => {
                self.apply_entity_update(api::TextSensorStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::NumberStateResponse => {
                self.apply_entity_update(api::NumberStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::SelectStateResponse => {
                self.apply_entity_update(api::SelectStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::SirenStateResponse => {
                self.apply_entity_update(api::SirenStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::LockStateResponse => {
                self.apply_entity_update(api::LockStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::MediaPlayerStateResponse => {
                self.apply_entity_update(api::MediaPlayerStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::AlarmControlPanelStateResponse => {
                self.apply_entity_update(api::AlarmControlPanelStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::TextStateResponse => {
                self.apply_entity_update(api::TextStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::DateStateResponse => {
                self.apply_entity_update(api::DateStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::TimeStateResponse => {
                self.apply_entity_update(api::TimeStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::ValveStateResponse => {
                self.apply_entity_update(api::ValveStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::DateTimeStateResponse => {
                self.apply_entity_update(api::DateTimeStateResponse::decode(msg)?)
                    .await?;
            }

            MessageType::UpdateStateResponse => {
                self.apply_entity_update(api::UpdateStateResponse::decode(msg)?)
                    .await?;
            }

            _ => {}
        }

        Ok(())
    }

    async fn apply_entity_update<T: EntityUpdate>(&self, update: T) -> Result<(), DeviceError> {
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
            .start_device_transaction(StartDeviceTransaction {
                device_idx: self.device_idx.unwrap(),
            })
            .await?;

        writer
            .select_entity(SelectEntity {
                entity_idx: *entity_idx,
            })
            .await?;

        update.write_to(&mut writer).await?;

        writer.end_transaction().await?;
        writer.flush().await?;

        Ok(())
    }
}

#[async_trait]
trait EntityUpdate {
    fn key(&self) -> u32;
    fn should_skip(&self) -> bool {
        false
    }
    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error>;
}

#[async_trait]
impl EntityUpdate for api::BinarySensorStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.bool(self.state).await
    }
}

#[async_trait]
impl EntityUpdate for api::CoverStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.position(self.position).await?;
        writer.tilt(self.tilt).await?;
        writer
            .cover_state(self.current_operation().as_igloo())
            .await
    }
}

#[async_trait]
impl EntityUpdate for api::FanStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.fan_speed(self.speed().as_igloo()).await?;
        writer.int(self.speed_level).await?;
        writer.fan_direction(self.direction().as_igloo()).await?;
        writer.text(self.preset_mode.clone()).await?;
        writer
            .fan_oscillation(match self.oscillating {
                true => FanOscillation::On,
                false => FanOscillation::Off,
            })
            .await
    }
}

#[async_trait]
impl EntityUpdate for api::LightStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .color(Color {
                r: (self.red * 255.) as u8,
                g: (self.green * 255.) as u8,
                b: (self.blue * 255.) as u8,
            })
            .await?;
        writer.dimmer(self.brightness).await?;
        writer.switch(self.state).await?;
        writer
            .color_temperature(self.color_temperature as u16)
            .await?;

        // ON_OFF = 1 << 0;
        // BRIGHTNESS = 1 << 1;
        // WHITE = 1 << 2;
        // COLOR_TEMPERATURE = 1 << 3;
        // COLD_WARM_WHITE = 1 << 4;
        // RGB = 1 << 5;

        // TODO FIXME is this right? Lowk i don't get the other ones

        if self.color_mode & (1 << 5) != 0 {
            writer.color_mode(ColorMode::RGB).await?;
        } else if self.color_mode & (1 << 3) != 0 {
            writer.color_mode(ColorMode::Temperature).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::SensorStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.float(self.state).await
    }
}

#[async_trait]
impl EntityUpdate for api::SwitchStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.switch(self.state).await
    }
}

#[async_trait]
impl EntityUpdate for api::TextSensorStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.text(self.state.clone()).await
    }
}

#[async_trait]
impl EntityUpdate for api::NumberStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.float(self.state).await
    }
}

#[async_trait]
impl EntityUpdate for api::SelectStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.text(self.state.clone()).await
    }
}

#[async_trait]
impl EntityUpdate for api::SirenStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.bool(self.state).await
    }
}

#[async_trait]
impl EntityUpdate for api::LockStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.lock_state(self.state().as_igloo()).await
    }
}

#[async_trait]
impl EntityUpdate for api::MediaPlayerStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.volume(self.volume).await?;
        writer.muted(self.muted).await?;
        writer.media_state(self.state().as_igloo()).await
    }
}

#[async_trait]
impl EntityUpdate for api::AlarmControlPanelStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.alarm_state(self.state().as_igloo()).await
    }
}

#[async_trait]
impl EntityUpdate for api::TextStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.text(self.state.clone()).await
    }
}

#[async_trait]
impl EntityUpdate for api::DateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .date(Date {
                year: self.year as u16, // FIXME make safe
                month: self.month as u8,
                day: self.day as u8,
            })
            .await
    }
}

#[async_trait]
impl EntityUpdate for api::TimeStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer
            .time(Time {
                hour: self.hour as u8,
                minute: self.minute as u8,
                second: self.second as u8,
            })
            .await
    }
}

#[async_trait]
impl EntityUpdate for api::ValveStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.position(self.position).await?;
        writer
            .valve_state(self.current_operation().as_igloo())
            .await
    }
}

#[async_trait]
impl EntityUpdate for api::DateTimeStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.date_time(self.epoch_seconds).await
    }
}

#[async_trait]
impl EntityUpdate for api::UpdateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn should_skip(&self) -> bool {
        self.missing_state
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        // FIXME I think the best way to handle update is by making
        // more entities for a clearer representation
        // But maybe this is good for reducing less used entities IDK

        let content = json!({
            "title": self.title,
            "current_version": self.current_version,
            "latest_version": self.latest_version,
            "release_summary": self.release_summary,
            "release_url": self.release_url
        });

        writer.bool(self.in_progress).await?;
        writer.text(content.to_string()).await?;

        if self.has_progress {
            writer.float(self.progress).await?;
        }

        Ok(())
    }
}

// --------------------------------------------

async fn add_entity_category(
    writer: &mut FloeWriterDefault,
    category: EntityCategory,
) -> Result<(), std::io::Error> {
    match category {
        EntityCategory::None => {}
        EntityCategory::Config => {
            writer.config().await?;
        }
        EntityCategory::Diagnostic => {
            writer.diagnostic().await?;
        }
    }
    Ok(())
}

async fn add_icon(writer: &mut FloeWriterDefault, icon: &str) -> Result<(), std::io::Error> {
    if !icon.is_empty() {
        writer.icon(icon.to_string()).await?;
    }
    Ok(())
}

async fn add_unit(writer: &mut FloeWriterDefault, unit_str: String) -> Result<(), std::io::Error> {
    // TODO log error, something else? if parsing failed..?
    if !unit_str.is_empty()
        && let Ok(unit) = Unit::try_from(unit_str)
    {
        writer.unit(unit).await?;
    }
    Ok(())
}

async fn add_f32_bounds(
    writer: &mut FloeWriterDefault,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> Result<(), std::io::Error> {
    writer.float_min(min).await?;
    writer.float_max(max).await?;
    if let Some(step) = step {
        writer.float_step(step).await?;
    }
    Ok(())
}

async fn add_device_class(
    writer: &mut FloeWriterDefault,
    device_class: String,
) -> Result<(), std::io::Error> {
    if !device_class.is_empty() {
        writer.device_class(device_class).await?;
    }
    Ok(())
}

async fn add_number_mode(
    writer: &mut FloeWriterDefault,
    number_mode: api::NumberMode,
) -> Result<(), std::io::Error> {
    writer.number_mode(number_mode.as_igloo()).await?;
    Ok(())
}

async fn add_climate_modes(
    writer: &mut FloeWriterDefault,
    modes: impl Iterator<Item = api::ClimateMode>,
) -> Result<(), std::io::Error> {
    let modes = modes.map(|m| m.as_igloo()).collect();
    writer.supported_climate_modes(modes).await?;
    Ok(())
}

async fn add_fan_speeds(
    writer: &mut FloeWriterDefault,
    modes: impl Iterator<Item = api::ClimateFanMode>,
) -> Result<(), std::io::Error> {
    let speeds = modes.map(|m| m.as_igloo()).collect();
    writer.supported_fan_speeds(speeds).await?;
    Ok(())
}

async fn add_fan_oscillations(
    writer: &mut FloeWriterDefault,
    modes: impl Iterator<Item = api::ClimateSwingMode>,
) -> Result<(), std::io::Error> {
    let modes = modes.map(|m| m.as_igloo()).collect();
    writer.supported_fan_oscillations(modes).await?;
    Ok(())
}

async fn add_sensor_state_class(
    writer: &mut FloeWriterDefault,
    state_class: api::SensorStateClass,
) -> Result<(), std::io::Error> {
    match state_class {
        api::SensorStateClass::StateClassNone => {}
        api::SensorStateClass::StateClassMeasurement => {
            writer
                .sensor_state_class(SensorStateClass::Measurement)
                .await?;
        }
        api::SensorStateClass::StateClassTotalIncreasing => {
            writer
                .sensor_state_class(SensorStateClass::TotalIncreasing)
                .await?;
        }
        api::SensorStateClass::StateClassTotal => {
            writer.sensor_state_class(SensorStateClass::Total).await?;
        }
    }
    Ok(())
}

impl api::NumberMode {
    fn as_igloo(&self) -> NumberMode {
        match self {
            api::NumberMode::Auto => NumberMode::Auto,
            api::NumberMode::Box => NumberMode::Box,
            api::NumberMode::Slider => NumberMode::Slider,
        }
    }
}

impl api::ClimateMode {
    fn as_igloo(&self) -> ClimateMode {
        match self {
            api::ClimateMode::Off => ClimateMode::Off,
            api::ClimateMode::HeatCool => ClimateMode::HeatCool,
            api::ClimateMode::Cool => ClimateMode::Cool,
            api::ClimateMode::Heat => ClimateMode::Heat,
            api::ClimateMode::FanOnly => ClimateMode::FanOnly,
            api::ClimateMode::Dry => ClimateMode::Dry,
            api::ClimateMode::Auto => ClimateMode::Auto,
        }
    }
}

impl api::ClimateFanMode {
    fn as_igloo(&self) -> FanSpeed {
        match self {
            api::ClimateFanMode::ClimateFanOn => FanSpeed::On,
            api::ClimateFanMode::ClimateFanOff => FanSpeed::Off,
            api::ClimateFanMode::ClimateFanAuto => FanSpeed::Auto,
            api::ClimateFanMode::ClimateFanLow => FanSpeed::Low,
            api::ClimateFanMode::ClimateFanMedium => FanSpeed::Medium,
            api::ClimateFanMode::ClimateFanHigh => FanSpeed::High,
            api::ClimateFanMode::ClimateFanMiddle => FanSpeed::Middle,
            api::ClimateFanMode::ClimateFanFocus => FanSpeed::Focus,
            api::ClimateFanMode::ClimateFanDiffuse => FanSpeed::Diffuse,
            api::ClimateFanMode::ClimateFanQuiet => FanSpeed::Quiet,
        }
    }
}

impl api::FanSpeed {
    fn as_igloo(&self) -> FanSpeed {
        match self {
            api::FanSpeed::Low => FanSpeed::Low,
            api::FanSpeed::Medium => FanSpeed::Medium,
            api::FanSpeed::High => FanSpeed::High,
        }
    }
}

impl api::ClimateSwingMode {
    fn as_igloo(&self) -> FanOscillation {
        match self {
            api::ClimateSwingMode::ClimateSwingOff => FanOscillation::Off,
            api::ClimateSwingMode::ClimateSwingBoth => FanOscillation::Both,
            api::ClimateSwingMode::ClimateSwingVertical => FanOscillation::Vertical,
            api::ClimateSwingMode::ClimateSwingHorizontal => FanOscillation::Horizontal,
        }
    }
}

impl api::TextMode {
    fn as_igloo(&self) -> TextMode {
        match self {
            api::TextMode::Text => TextMode::Text,
            api::TextMode::Password => TextMode::Password,
        }
    }
}

impl api::CoverOperation {
    fn as_igloo(&self) -> CoverState {
        match self {
            api::CoverOperation::Idle => CoverState::Idle,
            api::CoverOperation::IsOpening => CoverState::Opening,
            api::CoverOperation::IsClosing => CoverState::Closing,
        }
    }
}

impl api::FanDirection {
    fn as_igloo(&self) -> FanDirection {
        match self {
            api::FanDirection::Forward => FanDirection::Forward,
            api::FanDirection::Reverse => FanDirection::Reverse,
        }
    }
}

impl api::LockState {
    fn as_igloo(&self) -> LockState {
        match self {
            api::LockState::None => LockState::Unknown,
            api::LockState::Locked => LockState::Locked,
            api::LockState::Unlocked => LockState::Unlocked,
            api::LockState::Jammed => LockState::Jammed,
            api::LockState::Locking => LockState::Locking,
            api::LockState::Unlocking => LockState::Unlocking,
        }
    }
}

impl api::MediaPlayerState {
    fn as_igloo(&self) -> MediaState {
        match self {
            api::MediaPlayerState::None => MediaState::Unknown,
            api::MediaPlayerState::Idle => MediaState::Idle,
            api::MediaPlayerState::Playing => MediaState::Playing,
            api::MediaPlayerState::Paused => MediaState::Paused,
        }
    }
}

impl api::AlarmControlPanelState {
    fn as_igloo(&self) -> AlarmState {
        match self {
            api::AlarmControlPanelState::AlarmStateDisarmed => AlarmState::Disarmed,
            api::AlarmControlPanelState::AlarmStateArmedHome => AlarmState::ArmedHome,
            api::AlarmControlPanelState::AlarmStateArmedAway => AlarmState::ArmedAway,
            api::AlarmControlPanelState::AlarmStateArmedNight => AlarmState::ArmedNight,
            api::AlarmControlPanelState::AlarmStateArmedVacation => AlarmState::ArmedVacation,
            api::AlarmControlPanelState::AlarmStateArmedCustomBypass => AlarmState::ArmedUnknown,
            api::AlarmControlPanelState::AlarmStatePending => AlarmState::Pending,
            api::AlarmControlPanelState::AlarmStateArming => AlarmState::Arming,
            api::AlarmControlPanelState::AlarmStateDisarming => AlarmState::Disarming,
            api::AlarmControlPanelState::AlarmStateTriggered => AlarmState::Triggered,
        }
    }
}

impl api::ValveOperation {
    fn as_igloo(&self) -> ValveState {
        match self {
            api::ValveOperation::Idle => ValveState::Idle,
            api::ValveOperation::IsOpening => ValveState::Opening,
            api::ValveOperation::IsClosing => ValveState::Closing,
        }
    }
}

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
