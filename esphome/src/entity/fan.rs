use crate::{api, entity::EntityUpdate};
use async_trait::async_trait;
use igloo_interface::{FanDirection, FanOscillation, FanSpeed, FloeWriterDefault};
use super::{add_entity_category, add_icon, EntityRegister};

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesFanResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(writer, &self.name, self.key, crate::device::EntityType::Fan)
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
