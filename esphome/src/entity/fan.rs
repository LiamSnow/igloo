use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};
use async_trait::async_trait;
use igloo_interface::{
    DESELECT_ENTITY, END_TRANSACTION, FanDirection, FanOscillation, FanSpeed, FloeWriterDefault,
    WRITE_FAN_DIRECTION, WRITE_FAN_OSCILLATION, WRITE_INT, WRITE_SWITCH, WRITE_TEXT,
};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesFanResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::model::EntityType::Fan)
            .await?;
        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        Ok(())
    }
}

impl api::FanDirection {
    pub fn as_igloo(&self) -> FanDirection {
        match self {
            api::FanDirection::Forward => FanDirection::Forward,
            api::FanDirection::Reverse => FanDirection::Reverse,
        }
    }
}

impl api::FanSpeed {
    pub fn as_igloo(&self) -> FanSpeed {
        match self {
            api::FanSpeed::Low => FanSpeed::Low,
            api::FanSpeed::Medium => FanSpeed::Medium,
            api::FanSpeed::High => FanSpeed::High,
        }
    }
}

#[async_trait]
impl EntityUpdate for api::FanStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.fan_speed(&self.speed().as_igloo()).await?;
        writer.int(&self.speed_level).await?;
        writer.fan_direction(&self.direction().as_igloo()).await?;
        writer.text(&self.preset_mode.clone()).await?;
        writer
            .fan_oscillation(&match self.oscillating {
                true => FanOscillation::On,
                false => FanOscillation::Off,
            })
            .await
    }
}

fn fan_direction_to_api(direction: &FanDirection) -> api::FanDirection {
    match direction {
        FanDirection::Forward => api::FanDirection::Forward,
        FanDirection::Reverse => api::FanDirection::Reverse,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::FanCommandRequest {
        key,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_SWITCH => {
                let state: bool = borsh::from_slice(&payload)?;
                req.has_state = true;
                req.state = state;
            }

            WRITE_INT => {
                let speed_level: i32 = borsh::from_slice(&payload)?;
                req.has_speed_level = true;
                req.speed_level = speed_level;
            }

            WRITE_FAN_OSCILLATION => {
                let oscillation: FanOscillation = borsh::from_slice(&payload)?;
                req.has_oscillating = true;
                req.oscillating = match oscillation {
                    FanOscillation::Off => false,
                    _ => true,
                };
            }

            WRITE_FAN_DIRECTION => {
                let direction: FanDirection = borsh::from_slice(&payload)?;
                req.has_direction = true;
                req.direction = fan_direction_to_api(&direction).into();
            }

            WRITE_TEXT => {
                let preset: String = borsh::from_slice(&payload)?;
                req.has_preset_mode = true;
                req.preset_mode = preset;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Fan got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device.send_msg(MessageType::FanCommandRequest, &req).await
}
