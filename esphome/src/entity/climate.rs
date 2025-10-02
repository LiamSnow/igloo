use async_trait::async_trait;
use igloo_interface::{ClimateMode, FanOscillation, FanSpeed, FloeWriterDefault};

use super::{
    EntityRegister, add_climate_modes, add_entity_category, add_f32_bounds, add_fan_oscillations,
    add_fan_speeds, add_icon,
};
use crate::{api, entity::EntityUpdate};

// The ESPHome climate entity doesn't really match this ECS model
// So we are breaking it up into a few entities

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesClimateResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut igloo_interface::FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        // Humidity
        if self.supports_current_humidity || self.supports_target_humidity {
            let name = format!("{}_humidity", self.name);
            device
                .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                .await?;
            add_entity_category(writer, self.entity_category()).await?;
            add_icon(writer, &self.icon).await?;
            add_f32_bounds(
                writer,
                self.visual_min_humidity,
                self.visual_max_humidity,
                None,
            )
            .await?;
            writer.sensor().await?;
            writer.deselect_entity().await?;
        }

        // Current Temperature
        if self.supports_current_temperature {
            let name = format!("{}_current_temperature", self.name);
            device
                .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                .await?;
            add_entity_category(writer, self.entity_category()).await?;
            add_icon(writer, &self.icon).await?;
            add_f32_bounds(
                writer,
                self.visual_min_temperature,
                self.visual_max_temperature,
                Some(self.visual_current_temperature_step),
            )
            .await?;
            writer.sensor().await?;
            writer.deselect_entity().await?;
        }

        // Two Point Temperature
        if self.supports_two_point_target_temperature {
            // TODO verify this is right
            // Lower
            {
                let name = format!("{}_target_lower_temperature", self.name);
                device
                    .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                    .await?;
                add_entity_category(writer, self.entity_category()).await?;
                add_icon(writer, &self.icon).await?;
                add_f32_bounds(
                    writer,
                    self.visual_min_temperature,
                    self.visual_max_temperature,
                    Some(self.visual_target_temperature_step),
                )
                .await?;
                writer.deselect_entity().await?;
            }

            // Upper
            {
                let name = format!("{}_target_upper_temperature", self.name);
                device
                    .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                    .await?;
                add_entity_category(writer, self.entity_category()).await?;
                add_icon(writer, &self.icon).await?;
                add_f32_bounds(
                    writer,
                    self.visual_min_temperature,
                    self.visual_max_temperature,
                    Some(self.visual_target_temperature_step),
                )
                .await?;
                writer.deselect_entity().await?;
            }
        }
        // One Point Temperature Target
        else {
            let name = format!("{}_target_temperature", self.name);
            device
                .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                .await?;
            add_entity_category(writer, self.entity_category()).await?;
            add_icon(writer, &self.icon).await?;
            add_f32_bounds(
                writer,
                self.visual_min_temperature,
                self.visual_max_temperature,
                Some(self.visual_target_temperature_step),
            )
            .await?;
            writer.deselect_entity().await?;
        }

        // Climate Mode
        {
            let name = format!("{}_mode", self.name);
            device
                .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                .await?;
            add_entity_category(writer, self.entity_category()).await?;
            add_climate_modes(writer, self.supported_modes()).await?;
            add_icon(writer, &self.icon).await?;
            writer.deselect_entity().await?;
        }

        // Fan
        {
            let name = format!("{}_fan", self.name);
            device
                .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                .await?;
            add_entity_category(writer, self.entity_category()).await?;
            add_fan_speeds(writer, self.supported_fan_modes()).await?;
            add_fan_oscillations(writer, self.supported_swing_modes()).await?;
            add_icon(writer, &self.icon).await?;
            writer.deselect_entity().await?;
        }

        // Preset
        {
            let name = format!("{}_preset", self.name);
            device
                .register_entity(writer, &name, self.key, crate::device::EntityType::Climate)
                .await?;
            add_entity_category(writer, self.entity_category()).await?;
            add_icon(writer, &self.icon).await?;
            writer.text_select().await?;
            writer
                .text_list(
                    &self
                        .supported_presets()
                        .map(|preset| format!("{preset:#?}"))
                        .chain(self.supported_custom_presets.iter().cloned())
                        .collect(),
                )
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::ClimateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        // TODO FIXME plz implement!!!! :)
        println!("ERROR CLIMATE HAS NOT BEEN IMPLEMENTED");

        Ok(())
    }
}

impl api::ClimateSwingMode {
    pub fn as_igloo(&self) -> FanOscillation {
        match self {
            api::ClimateSwingMode::ClimateSwingOff => FanOscillation::Off,
            api::ClimateSwingMode::ClimateSwingBoth => FanOscillation::Both,
            api::ClimateSwingMode::ClimateSwingVertical => FanOscillation::Vertical,
            api::ClimateSwingMode::ClimateSwingHorizontal => FanOscillation::Horizontal,
        }
    }
}
impl api::ClimateMode {
    pub fn as_igloo(&self) -> ClimateMode {
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
    pub fn as_igloo(&self) -> FanSpeed {
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
