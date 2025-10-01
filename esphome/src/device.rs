use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::BytesMut;
use igloo_interface::{
    AccuracyDecimals, AlarmState, Bool, ClimateMode, Color, ColorMode, ColorTemperature, Component,
    CoverState, Date, DateTime, DeviceClass, Diagnostic, Dimmer, FanDirection, FanOscillation,
    FanSpeed, Float, FloatMax, FloatMin, FloatStep, Icon, Int, LockState, MediaState, Muted,
    NumberMode, Position, RequestUpdatesPayload, SensorStateClass, SupportedClimateModes,
    SupportedFanOscillations, SupportedFanSpeeds, Switch, Text, TextList, TextMaxLength,
    TextMinLength, TextMode, TextPattern, TextSelect, Tilt, Time, UpdatesPayload, ValveState,
    Volume,
    floe::{FloeManager, FloeManagerError},
};
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::sync::mpsc;

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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConnectionParams {
    pub ip: String,
    pub noise_psk: Option<String>,
    pub password: Option<String>,
}

pub struct Device {
    pub connection: Connection,
    pub password: String,
    pub connected: bool,
    pub last_ping: Option<SystemTime>,
    pub manager: FloeManager,
    /// maps ESPHome entity key -> Igloo entity ephermeral ref
    pub entity_key_to_ref: HashMap<u32, u16>,
    /// maps Igloo entity ephermeral ref -> ESPHome type,key
    pub entity_ref_to_info: HashMap<u16, (EntityType, u32)>,
    pub device_ref: Option<u16>,
}

pub struct EntityData {
    pub name: String,
    key: u32,
    typ: EntityType,
    components: Vec<Component>,
}

#[derive(Error, Debug)]
pub enum DeviceError {
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
    #[error("floe manager error `{0}`")]
    FloeManager(#[from] FloeManagerError),
}

impl Device {
    pub fn new(params: ConnectionParams, manager: FloeManager) -> Self {
        let connection = match params.noise_psk {
            Some(noise_psk) => NoiseConnection::new(params.ip, noise_psk).into(),
            None => PlainConnection::new(params.ip).into(),
        };

        Device {
            connection,
            password: params.password.unwrap_or_default(),
            connected: false,
            last_ping: None,
            manager,
            device_ref: None,
            entity_key_to_ref: HashMap::new(),
            entity_ref_to_info: HashMap::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<Vec<EntityData>, DeviceError> {
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

        self.get_entities().await
    }

    async fn subscribe_states(&mut self) -> Result<(), DeviceError> {
        self.recv_process_msg().await?;
        self.send_msg(
            MessageType::SubscribeStatesRequest,
            &api::SubscribeStatesRequest {},
        )
        .await?;

        Ok(())
    }

    /// Send disconnect request to device, wait for response, then disconnect socket
    pub async fn disconnect(&mut self) -> Result<(), DeviceError> {
        self.recv_process_msg().await?;
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
        self.recv_process_msg().await?;
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

    /// after Igloo confirmed our device registration we can
    ///  1. make our entitiy ref mappings
    ///  2. send initial components
    pub async fn init(
        &mut self,
        device_ref: u16,
        entity_refs: Vec<u16>,
        entities: Vec<EntityData>,
    ) -> Result<(), DeviceError> {
        self.device_ref = Some(device_ref);

        for (entity_ref, entity) in entity_refs.into_iter().zip(entities.into_iter()) {
            self.entity_key_to_ref.insert(entity.key, entity_ref);
            self.entity_ref_to_info
                .insert(entity_ref, (entity.typ, entity.key));

            self.manager
                .updates(UpdatesPayload {
                    device: device_ref,
                    entity: entity_ref,
                    values: entity.components,
                })
                .await?;
        }

        Ok(())
    }

    pub async fn run(
        mut self,
        mut cmd_rx: mpsc::Receiver<RequestUpdatesPayload>,
    ) -> Result<(), DeviceError> {
        self.subscribe_states().await?;

        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(cmd) => {
                            if let Err(_) = self.handle_cmd(cmd).await {
                                // TODO: handle error
                            }
                        }
                        None => {
                            // TODO return device disconnected err
                            return Ok(())
                        }
                    }
                }

                result = self.connection.readable() => if result.is_ok() {
                    if let Err(e) = self.recv_process_msg().await {
                        if matches!(e, DeviceError::DeviceRequestShutdown) {
                            return Ok(());
                        }
                    }
                }
            }
        }
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
                if let Some(device_ref) = self.device_ref {
                    self.process_state_update(msg_type, msg, device_ref).await?;
                }
                // TODO else log?
            }
        }
        Ok(())
    }

    async fn process_state_update(
        &mut self,
        msg_type: MessageType,
        msg: BytesMut,
        device: u16,
    ) -> Result<(), DeviceError> {
        let (key, values) = match msg_type {
            MessageType::DisconnectRequest
            | MessageType::PingRequest
            | MessageType::PingResponse
            | MessageType::GetTimeRequest
            | MessageType::SubscribeLogsResponse => {
                unreachable!()
            }

            MessageType::BinarySensorStateResponse => {
                let update = api::BinarySensorStateResponse::decode(msg)?;
                // TODO should missing state be a component or something or is
                // ignoring it like this valid?
                if update.missing_state {
                    return Ok(());
                }
                (update.key, vec![Component::Bool(Bool(update.state))])
            }

            MessageType::CoverStateResponse => {
                let update = api::CoverStateResponse::decode(msg)?;
                (
                    update.key,
                    vec![
                        Component::Position(Position(update.position)),
                        Component::Tilt(Tilt(update.tilt)),
                        Component::CoverState(update.current_operation().as_igloo()),
                    ],
                )
            }

            MessageType::FanStateResponse => {
                let update = api::FanStateResponse::decode(msg)?;

                (
                    update.key,
                    vec![
                        Component::FanSpeed(update.speed().as_igloo()),
                        Component::Int(Int(update.speed_level)),
                        Component::FanDirection(update.direction().as_igloo()),
                        Component::Text(Text(update.preset_mode)),
                        Component::FanOscillation(match update.oscillating {
                            true => FanOscillation::On,
                            false => FanOscillation::Off,
                        }),
                    ],
                )
            }

            MessageType::LightStateResponse => {
                let update = api::LightStateResponse::decode(msg)?;
                // TODO probably should be using supported modes here
                let mut values = vec![
                    Component::Color(Color {
                        r: (update.red * 255.) as u8,
                        g: (update.green * 255.) as u8,
                        b: (update.blue * 255.) as u8,
                    }),
                    Component::Dimmer(Dimmer(update.brightness)),
                    Component::Switch(Switch(update.state)),
                    Component::ColorTemperature(ColorTemperature(update.color_temperature as u16)),
                ];

                // ON_OFF = 1 << 0;
                // BRIGHTNESS = 1 << 1;
                // WHITE = 1 << 2;
                // COLOR_TEMPERATURE = 1 << 3;
                // COLD_WARM_WHITE = 1 << 4;
                // RGB = 1 << 5;

                // TODO FIXME is this right? Lowk i don't get the other ones

                if update.color_mode & (1 << 5) != 0 {
                    values.push(Component::ColorMode(ColorMode::RGB));
                } else if update.color_mode & (1 << 3) != 0 {
                    values.push(Component::ColorMode(ColorMode::Temperature))
                }

                (update.key, values)
            }

            MessageType::SensorStateResponse => {
                let update = api::SensorStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }
                (update.key, vec![Component::Float(Float(update.state))])
            }

            MessageType::SwitchStateResponse => {
                let update = api::SwitchStateResponse::decode(msg)?;
                (update.key, vec![Component::Switch(Switch(update.state))])
            }

            MessageType::TextSensorStateResponse => {
                let update = api::TextSensorStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }
                (update.key, vec![Component::Text(Text(update.state))])
            }

            // TODO
            // MessageType::ClimateStateResponse => {
            //     let update = api::ClimateStateResponse::decode(msg)?;
            //     updates.push(ComponentUpdate {
            //         device: self.device_id,
            //         entity: entity_name.to_string(),
            //         values: todo!(),
            //     });
            // }
            MessageType::NumberStateResponse => {
                let update = api::NumberStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }
                (update.key, vec![Component::Float(Float(update.state))])
            }

            MessageType::SelectStateResponse => {
                let update = api::SelectStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }
                (update.key, vec![Component::Text(Text(update.state))])
            }

            MessageType::SirenStateResponse => {
                let update = api::SirenStateResponse::decode(msg)?;
                // TODO should this be a bool or something else?
                // Maybe the sensor component should take a bool?
                (update.key, vec![Component::Bool(Bool(update.state))])
            }

            MessageType::LockStateResponse => {
                let update = api::LockStateResponse::decode(msg)?;
                (
                    update.key,
                    vec![Component::LockState(update.state().as_igloo())],
                )
            }

            MessageType::MediaPlayerStateResponse => {
                let update = api::MediaPlayerStateResponse::decode(msg)?;
                (
                    update.key,
                    vec![
                        Component::Volume(Volume(update.volume)),
                        Component::Muted(Muted(update.muted)),
                        Component::MediaState(update.state().as_igloo()),
                    ],
                )
            }

            MessageType::AlarmControlPanelStateResponse => {
                let update = api::AlarmControlPanelStateResponse::decode(msg)?;
                (
                    update.key,
                    vec![Component::AlarmState(update.state().as_igloo())],
                )
            }

            MessageType::TextStateResponse => {
                let update = api::TextStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }
                (update.key, vec![Component::Text(Text(update.state))])
            }

            MessageType::DateStateResponse => {
                let update = api::DateStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }
                (
                    update.key,
                    vec![Component::Date(Date {
                        year: update.year as u16, // FIXME make safe
                        month: update.month as u8,
                        day: update.day as u8,
                    })],
                )
            }

            MessageType::TimeStateResponse => {
                let update = api::TimeStateResponse::decode(msg)?;
                (
                    update.key,
                    vec![Component::Time(Time {
                        hour: update.hour as u8,
                        minute: update.minute as u8,
                        second: update.second as u8,
                    })],
                )
            }

            MessageType::ValveStateResponse => {
                let update = api::ValveStateResponse::decode(msg)?;
                (
                    update.key,
                    vec![
                        Component::Position(Position(update.position)),
                        Component::ValveState(update.current_operation().as_igloo()),
                    ],
                )
            }

            MessageType::DateTimeStateResponse => {
                let update = api::DateTimeStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }

                (
                    update.key,
                    vec![Component::DateTime(DateTime(update.epoch_seconds))],
                )
            }

            MessageType::UpdateStateResponse => {
                let update = api::UpdateStateResponse::decode(msg)?;
                if update.missing_state {
                    return Ok(());
                }

                // FIXME I think the best way to handle update is by making
                // more entities for a clearer representation
                // But maybe this is good for reducing less used entities IDK

                let content = json!({
                    "title": update.title,
                    "current_version": update.current_version,
                    "latest_version": update.latest_version,
                    "release_summary": update.release_summary,
                    "release_url": update.release_url
                });

                let mut values = vec![
                    Component::Bool(Bool(update.in_progress)),
                    Component::Text(Text(content.to_string())),
                ];

                if update.has_progress {
                    values.push(Component::Float(Float(update.progress)));
                }

                (update.key, values)
            }

            _ => {
                panic!() // FIXME
                // TODO log unknown msgs?
            }
        };

        let Some(entity_ref) = self.entity_key_to_ref.get(&key) else {
            // TODO what the hell is this? Why is it giving an update
            // for an unknown entity. Should we be failing here??
            return Ok(());
        };

        self.manager
            .updates(UpdatesPayload {
                device,
                entity: *entity_ref,
                values,
            })
            .await?;

        Ok(())
    }

    pub async fn get_entities(&mut self) -> Result<Vec<EntityData>, DeviceError> {
        self.recv_process_msg().await?;
        self.send_msg(
            MessageType::ListEntitiesRequest,
            &api::ListEntitiesRequest {},
        )
        .await?;

        let mut entities = Vec::new();

        loop {
            let (msg_type, msg) = self.connection.recv_msg().await?;
            use EntityType::*;

            match msg_type {
                MessageType::ListEntitiesServicesResponse => {
                    // TODO should we support services?
                }
                MessageType::ListEntitiesDoneResponse => break,

                MessageType::ListEntitiesBinarySensorResponse => {
                    let msg = api::ListEntitiesBinarySensorResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(4);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    components.push(Component::Sensor(igloo_interface::Sensor));
                    entities.push(EntityData::new(msg.name, msg.key, BinarySensor, components));
                }

                MessageType::ListEntitiesCoverResponse => {
                    let msg = api::ListEntitiesCoverResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(4);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    components.push(Component::Sensor(igloo_interface::Sensor));
                    entities.push(EntityData::new(msg.name, msg.key, Cover, components));
                }

                MessageType::ListEntitiesFanResponse => {
                    let msg = api::ListEntitiesFanResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    entities.push(EntityData::new(msg.name, msg.key, Fan, components));
                }

                MessageType::ListEntitiesLightResponse => {
                    let msg = api::ListEntitiesLightResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    entities.push(EntityData::new(msg.name, msg.key, Light, components));
                }

                MessageType::ListEntitiesSensorResponse => {
                    let msg = api::ListEntitiesSensorResponse::decode(msg)?;

                    let mut components = Vec::with_capacity(7);
                    add_entity_category(&mut components, msg.entity_category());
                    add_sensor_state_class(&mut components, msg.state_class());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    add_unit(&mut components, msg.unit_of_measurement);
                    components.push(Component::Sensor(igloo_interface::Sensor));
                    components.push(Component::AccuracyDecimals(AccuracyDecimals(
                        msg.accuracy_decimals,
                    )));
                    entities.push(EntityData::new(msg.name, msg.key, Sensor, components));
                }

                MessageType::ListEntitiesSwitchResponse => {
                    let msg = api::ListEntitiesSwitchResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(3);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    entities.push(EntityData::new(msg.name, msg.key, Switch, components));
                }

                MessageType::ListEntitiesTextSensorResponse => {
                    let msg = api::ListEntitiesTextSensorResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(4);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    components.push(Component::Sensor(igloo_interface::Sensor));
                    entities.push(EntityData::new(msg.name, msg.key, TextSensor, components));
                }

                MessageType::ListEntitiesCameraResponse => {
                    let msg = api::ListEntitiesCameraResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    entities.push(EntityData::new(msg.name, msg.key, Camera, components));
                }

                // The ESPHome climate entity doesn't really match this ECS model
                // So we are breaking it up into a few entities
                MessageType::ListEntitiesClimateResponse => {
                    let msg = api::ListEntitiesClimateResponse::decode(msg)?;

                    // Humidity
                    if msg.supports_current_humidity || msg.supports_target_humidity {
                        let name = format!("{}_humidity", msg.name);
                        let mut components = Vec::with_capacity(4);
                        add_entity_category(&mut components, msg.entity_category());
                        add_icon(&mut components, &msg.icon);
                        add_f32_bounds(
                            &mut components,
                            msg.visual_min_humidity,
                            msg.visual_max_humidity,
                            None,
                        );
                        components.push(Component::Sensor(igloo_interface::Sensor));
                        entities.push(EntityData::new(name, msg.key, Climate, components));
                    }

                    // Current Temperature
                    if msg.supports_current_temperature {
                        let name = format!("{}_current_temperature", msg.name);
                        let mut components = Vec::with_capacity(4);
                        add_entity_category(&mut components, msg.entity_category());
                        add_icon(&mut components, &msg.icon);
                        add_f32_bounds(
                            &mut components,
                            msg.visual_min_temperature,
                            msg.visual_max_temperature,
                            Some(msg.visual_current_temperature_step),
                        );
                        components.push(Component::Sensor(igloo_interface::Sensor));
                        entities.push(EntityData::new(name, msg.key, Climate, components));
                    }

                    // Two Point Temperature
                    if msg.supports_two_point_target_temperature {
                        // TODO verify this is right
                        // Lower
                        {
                            let name = format!("{}_target_lower_temperature", msg.name);
                            let mut components = Vec::with_capacity(3);
                            add_entity_category(&mut components, msg.entity_category());
                            add_icon(&mut components, &msg.icon);
                            add_f32_bounds(
                                &mut components,
                                msg.visual_min_temperature,
                                msg.visual_max_temperature,
                                Some(msg.visual_target_temperature_step),
                            );
                            entities.push(EntityData::new(name, msg.key, Climate, components));
                        }

                        // Upper
                        {
                            let name = format!("{}_target_upper_temperature", msg.name);
                            let mut components = Vec::with_capacity(3);
                            add_entity_category(&mut components, msg.entity_category());
                            add_icon(&mut components, &msg.icon);
                            add_f32_bounds(
                                &mut components,
                                msg.visual_min_temperature,
                                msg.visual_max_temperature,
                                Some(msg.visual_target_temperature_step),
                            );
                            entities.push(EntityData::new(name, msg.key, Climate, components));
                        }
                    }
                    // One Point Temperature Target
                    else {
                        let name = format!("{}_target_temperature", msg.name);
                        let mut components = Vec::with_capacity(3);
                        add_entity_category(&mut components, msg.entity_category());
                        add_icon(&mut components, &msg.icon);
                        add_f32_bounds(
                            &mut components,
                            msg.visual_min_temperature,
                            msg.visual_max_temperature,
                            Some(msg.visual_target_temperature_step),
                        );
                        entities.push(EntityData::new(name, msg.key, Climate, components));
                    }

                    // Climate Mode
                    {
                        let name = format!("{}_mode", msg.name);
                        let mut components = Vec::with_capacity(3);
                        add_entity_category(&mut components, msg.entity_category());
                        add_climate_modes(&mut components, msg.supported_modes());
                        add_icon(&mut components, &msg.icon);
                        entities.push(EntityData::new(name, msg.key, Climate, components));
                    }

                    // Fan
                    {
                        let name = format!("{}_fan", msg.name);
                        let mut components = Vec::with_capacity(4);
                        add_entity_category(&mut components, msg.entity_category());
                        add_fan_speeds(&mut components, msg.supported_fan_modes());
                        add_fan_oscillations(&mut components, msg.supported_swing_modes());
                        add_icon(&mut components, &msg.icon);
                        entities.push(EntityData::new(name, msg.key, Climate, components));
                    }
                    // preset
                    {
                        let name = format!("{}_preset", msg.name);
                        let mut components = Vec::with_capacity(4);
                        add_entity_category(&mut components, msg.entity_category());
                        add_icon(&mut components, &msg.icon);
                        components.push(Component::TextSelect(TextSelect));
                        components.push(Component::TextList(TextList(
                            msg.supported_presets()
                                .map(|preset| format!("{preset:#?}"))
                                .chain(msg.supported_custom_presets.iter().cloned())
                                .collect(),
                        )));
                        entities.push(EntityData::new(name, msg.key, Climate, components));
                    }
                }

                MessageType::ListEntitiesNumberResponse => {
                    let msg = api::ListEntitiesNumberResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(6);
                    add_entity_category(&mut components, msg.entity_category());
                    add_number_mode(&mut components, msg.mode());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    add_f32_bounds(
                        &mut components,
                        msg.min_value,
                        msg.max_value,
                        Some(msg.step),
                    );
                    add_unit(&mut components, msg.unit_of_measurement);
                    entities.push(EntityData::new(msg.name, msg.key, Number, components));
                }

                MessageType::ListEntitiesSelectResponse => {
                    let msg = api::ListEntitiesSelectResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(4);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    components.push(Component::TextSelect(TextSelect));
                    components.push(Component::TextList(TextList(msg.options)));
                    entities.push(EntityData::new(msg.name, msg.key, Select, components));
                }

                MessageType::ListEntitiesSirenResponse => {
                    let msg = api::ListEntitiesSirenResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(5);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    components.push(Component::TextSelect(TextSelect));
                    components.push(Component::TextList(TextList(msg.tones)));
                    components.push(Component::Siren(igloo_interface::Siren));
                    entities.push(EntityData::new(msg.name, msg.key, Siren, components));
                }

                MessageType::ListEntitiesLockResponse => {
                    let msg = api::ListEntitiesLockResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(3);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    components.push(Component::Text(igloo_interface::Text(msg.code_format))); // TODO is this right?
                    entities.push(EntityData::new(msg.name, msg.key, Lock, components));
                }

                MessageType::ListEntitiesButtonResponse => {
                    let msg = api::ListEntitiesButtonResponse::decode(msg)?;
                    // TODO should this have a Button component or something?
                    // to produce an event?
                    let mut components = Vec::with_capacity(3);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    entities.push(EntityData::new(msg.name, msg.key, Button, components));
                }

                MessageType::ListEntitiesMediaPlayerResponse => {
                    let msg = api::ListEntitiesMediaPlayerResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    // TODO .supported_formats
                    entities.push(EntityData::new(msg.name, msg.key, MediaPlayer, components));
                }

                MessageType::ListEntitiesAlarmControlPanelResponse => {
                    let msg = api::ListEntitiesAlarmControlPanelResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    // TODO .supported_features
                    entities.push(EntityData::new(
                        msg.name,
                        msg.key,
                        AlarmControlPanel,
                        components,
                    ));
                }

                MessageType::ListEntitiesTextResponse => {
                    let msg = api::ListEntitiesTextResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(6);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    components.push(Component::TextMode(msg.mode().as_igloo()));
                    components.push(Component::TextMinLength(TextMinLength(msg.min_length)));
                    components.push(Component::TextMaxLength(TextMaxLength(msg.max_length)));
                    components.push(Component::TextPattern(TextPattern(msg.pattern)));
                    entities.push(EntityData::new(msg.name, msg.key, Text, components));
                }

                MessageType::ListEntitiesDateResponse => {
                    let msg = api::ListEntitiesDateResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    entities.push(EntityData::new(msg.name, msg.key, Date, components));
                }

                MessageType::ListEntitiesTimeResponse => {
                    let msg = api::ListEntitiesTimeResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    entities.push(EntityData::new(msg.name, msg.key, Time, components));
                }

                MessageType::ListEntitiesEventResponse => {
                    let msg = api::ListEntitiesEventResponse::decode(msg)?;
                    // TODO should this have an event component?
                    let mut components = Vec::with_capacity(4);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    components.push(Component::TextList(TextList(msg.event_types)));
                    entities.push(EntityData::new(msg.name, msg.key, Event, components));
                }

                MessageType::ListEntitiesValveResponse => {
                    let msg = api::ListEntitiesValveResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(4);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    components.push(Component::Valve(igloo_interface::Valve));
                    // TODO supports position/stop?
                    entities.push(EntityData::new(msg.name, msg.key, Valve, components));
                }

                MessageType::ListEntitiesDateTimeResponse => {
                    let msg = api::ListEntitiesDateTimeResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(2);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    entities.push(EntityData::new(msg.name, msg.key, DateTime, components));
                }

                MessageType::ListEntitiesUpdateResponse => {
                    let msg = api::ListEntitiesUpdateResponse::decode(msg)?;
                    let mut components = Vec::with_capacity(3);
                    add_entity_category(&mut components, msg.entity_category());
                    add_icon(&mut components, &msg.icon);
                    add_device_class(&mut components, msg.device_class);
                    entities.push(EntityData::new(msg.name, msg.key, Update, components));
                }

                _ => {}
            }
        }

        Ok(entities)
    }

    pub async fn handle_cmd(&mut self, cmd: RequestUpdatesPayload) -> Result<(), DeviceError> {
        let (typ, key) = self
            .entity_ref_to_info
            .get(&cmd.entity)
            .ok_or(DeviceError::InvalidEntity(cmd.entity))?;

        match typ {
            // TODO all
            EntityType::BinarySensor => todo!(),
            EntityType::Cover => todo!(),
            EntityType::Fan => todo!(),
            EntityType::Light => {
                let mut res = LightCommandRequest {
                    key: *key,
                    ..Default::default()
                };

                for value in cmd.values {
                    match value {
                        Component::Color(color) => {
                            res.has_rgb = true;
                            res.red = (color.r as f32) / 255.;
                            res.green = (color.g as f32) / 255.;
                            res.blue = (color.b as f32) / 255.;
                        }
                        Component::Dimmer(Dimmer(val)) => {
                            res.has_color_brightness = true;
                            res.color_brightness = val;
                            res.has_brightness = true;
                            res.brightness = val;
                        }
                        Component::Switch(Switch(state)) => {
                            res.has_state = true;
                            res.state = state;
                        }
                        Component::ColorTemperature(ColorTemperature(temp)) => {
                            res.has_color_temperature = true;
                            res.color_temperature = temp as f32;
                        }
                        Component::ColorMode(mode) => {
                            res.has_color_mode = true;
                            res.color_mode = match mode {
                                ColorMode::RGB => 35,
                                ColorMode::Temperature => 11,
                                ColorMode::Custom(_) => panic!(), // FIXME
                            };
                        }
                        _ => {} // TODO send error?
                    }
                }

                self.send_msg(MessageType::LightCommandRequest, &res)
                    .await?;
            }
            // TODO all
            EntityType::Sensor => todo!(),
            EntityType::Switch => todo!(),
            EntityType::TextSensor => todo!(),
            EntityType::Camera => todo!(),
            EntityType::Climate => todo!(),
            EntityType::Number => todo!(),
            EntityType::Select => todo!(),
            EntityType::Siren => todo!(),
            EntityType::Lock => todo!(),
            EntityType::Button => todo!(),
            EntityType::MediaPlayer => todo!(),
            EntityType::AlarmControlPanel => todo!(),
            EntityType::Text => todo!(),
            EntityType::Date => todo!(),
            EntityType::Time => todo!(),
            EntityType::Event => todo!(),
            EntityType::Valve => todo!(),
            EntityType::DateTime => todo!(),
            EntityType::Update => todo!(),
        }

        Ok(())
    }
}

impl EntityData {
    fn new(name: String, key: u32, typ: EntityType, components: Vec<Component>) -> Self {
        Self {
            name,
            key,
            typ,
            components,
        }
    }
}

fn add_entity_category(values: &mut Vec<Component>, category: EntityCategory) {
    match category {
        EntityCategory::None => {}
        EntityCategory::Config => {
            values.push(Component::Config(igloo_interface::Config));
        }
        EntityCategory::Diagnostic => {
            values.push(Component::Diagnostic(Diagnostic));
        }
    }
}

fn add_icon(values: &mut Vec<Component>, icon: &str) {
    if !icon.is_empty() {
        values.push(Component::Icon(Icon(icon.to_string())));
    }
}

fn add_unit(values: &mut Vec<Component>, unit: String) {
    if !unit.is_empty() {
        values.push(Component::Unit(unit.into()));
    }
}

fn add_f32_bounds(values: &mut Vec<Component>, min: f32, max: f32, step: Option<f32>) {
    values.push(Component::FloatMin(FloatMin(min)));
    values.push(Component::FloatMax(FloatMax(max)));
    if let Some(step) = step {
        values.push(Component::FloatStep(FloatStep(step)));
    }
}

fn add_device_class(values: &mut Vec<Component>, device_class: String) {
    if !device_class.is_empty() {
        values.push(Component::DeviceClass(DeviceClass(device_class)));
    }
}

fn add_number_mode(values: &mut Vec<Component>, number_mode: api::NumberMode) {
    values.push(Component::NumberMode(number_mode.as_igloo()));
}

fn add_climate_modes(values: &mut Vec<Component>, modes: impl Iterator<Item = api::ClimateMode>) {
    let modes = modes.map(|m| m.as_igloo()).collect();
    values.push(Component::SupportedClimateModes(SupportedClimateModes(
        modes,
    )));
}

fn add_fan_speeds(values: &mut Vec<Component>, modes: impl Iterator<Item = api::ClimateFanMode>) {
    let speeds = modes.map(|m| m.as_igloo()).collect();
    values.push(Component::SupportedFanSpeeds(SupportedFanSpeeds(speeds)));
}

fn add_fan_oscillations(
    values: &mut Vec<Component>,
    modes: impl Iterator<Item = api::ClimateSwingMode>,
) {
    let modes = modes.map(|m| m.as_igloo()).collect();
    values.push(Component::SupportedFanOscillations(
        SupportedFanOscillations(modes),
    ));
}

fn add_sensor_state_class(values: &mut Vec<Component>, state_class: api::SensorStateClass) {
    match state_class {
        api::SensorStateClass::StateClassNone => {}
        api::SensorStateClass::StateClassMeasurement => {
            values.push(Component::SensorStateClass(SensorStateClass::Measurement));
        }
        api::SensorStateClass::StateClassTotalIncreasing => {
            values.push(Component::SensorStateClass(
                SensorStateClass::TotalIncreasing,
            ));
        }
        api::SensorStateClass::StateClassTotal => {
            values.push(Component::SensorStateClass(SensorStateClass::Total));
        }
    }
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
