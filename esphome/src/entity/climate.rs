use async_trait::async_trait;
use igloo_interface::{
    ClimateMode, DESELECT_ENTITY, END_TRANSACTION, FanOscillation, FanSpeed, Real, Text,
    WRITE_CLIMATE_MODE, WRITE_FAN_OSCILLATION, WRITE_FAN_SPEED, WRITE_REAL, WRITE_TEXT,
    floe::FloeWriterDefault,
};

use super::{EntityRegister, add_entity_category, add_icon};
use crate::{
    api,
    device::{Device, DeviceError},
    entity::EntityUpdate,
    model::MessageType,
};

// The ESPHome climate entity doesn't really match this ECS model
// Currently we aren't publishing Humidity or Cur Temp
// The best way to do this is probably by splitting into more entities
// but I didn't really setup device.rs to handle that properly bc
// its annoying.

#[async_trait]
impl EntityRegister for crate::api::ListEntitiesClimateResponse {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError> {
        device
            .register_entity(
                writer,
                &self.name,
                self.key,
                crate::model::EntityType::Climate,
            )
            .await?;

        add_entity_category(writer, self.entity_category()).await?;
        add_icon(writer, &self.icon).await?;
        // add_f32_bounds(
        //     writer,
        //     self.visual_min_temperature,
        //     self.visual_max_temperature,
        //     Some(self.visual_target_temperature_step),
        // )
        // .await?;

        // add_climate_modes(writer, self.supported_modes()).await?;

        // add_fan_speeds(writer, self.supported_fan_modes()).await?;
        // add_fan_oscillations(writer, self.supported_swing_modes()).await?;

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

        Ok(())
    }
}

#[async_trait]
impl EntityUpdate for api::ClimateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error> {
        writer.real(&(self.target_temperature as f64)).await?;

        writer.climate_mode(&self.mode().as_igloo()).await?;

        writer.fan_speed(&self.fan_mode().as_igloo()).await?;
        writer
            .fan_oscillation(&self.swing_mode().as_igloo())
            .await?;

        writer.text(&format!("{:#?}", self.preset())).await?;
        writer.text(&self.custom_preset).await?;

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

fn climate_mode_to_api(mode: &ClimateMode) -> api::ClimateMode {
    match mode {
        ClimateMode::Off => api::ClimateMode::Off,
        ClimateMode::HeatCool => api::ClimateMode::HeatCool,
        ClimateMode::Cool => api::ClimateMode::Cool,
        ClimateMode::Heat => api::ClimateMode::Heat,
        ClimateMode::FanOnly => api::ClimateMode::FanOnly,
        ClimateMode::Dry => api::ClimateMode::Dry,
        ClimateMode::Auto => api::ClimateMode::Auto,
        ClimateMode::Eco => api::ClimateMode::Auto,
    }
}

fn fan_speed_to_climate_fan(speed: &FanSpeed) -> api::ClimateFanMode {
    match speed {
        FanSpeed::On => api::ClimateFanMode::ClimateFanOn,
        FanSpeed::Off => api::ClimateFanMode::ClimateFanOff,
        FanSpeed::Auto => api::ClimateFanMode::ClimateFanAuto,
        FanSpeed::Low => api::ClimateFanMode::ClimateFanLow,
        FanSpeed::Medium => api::ClimateFanMode::ClimateFanMedium,
        FanSpeed::High => api::ClimateFanMode::ClimateFanHigh,
        FanSpeed::Middle => api::ClimateFanMode::ClimateFanMiddle,
        FanSpeed::Focus => api::ClimateFanMode::ClimateFanFocus,
        FanSpeed::Diffuse => api::ClimateFanMode::ClimateFanDiffuse,
        FanSpeed::Quiet => api::ClimateFanMode::ClimateFanQuiet,
    }
}

fn fan_oscillation_to_swing(oscillation: &FanOscillation) -> api::ClimateSwingMode {
    match oscillation {
        FanOscillation::Off => api::ClimateSwingMode::ClimateSwingOff,
        FanOscillation::On => api::ClimateSwingMode::ClimateSwingBoth,
        FanOscillation::Vertical => api::ClimateSwingMode::ClimateSwingVertical,
        FanOscillation::Horizontal => api::ClimateSwingMode::ClimateSwingHorizontal,
        FanOscillation::Both => api::ClimateSwingMode::ClimateSwingBoth,
    }
}

pub async fn process(
    device: &mut Device,
    key: u32,
    commands: Vec<(u16, Vec<u8>)>,
) -> Result<(), DeviceError> {
    let mut req = api::ClimateCommandRequest {
        key,
        ..Default::default()
    };

    for (cmd_id, payload) in commands {
        match cmd_id {
            WRITE_CLIMATE_MODE => {
                let mode: ClimateMode = borsh::from_slice(&payload)?;
                req.has_mode = true;
                req.mode = climate_mode_to_api(&mode).into();
            }

            WRITE_FAN_SPEED => {
                let speed: FanSpeed = borsh::from_slice(&payload)?;
                req.has_fan_mode = true;
                req.fan_mode = fan_speed_to_climate_fan(&speed).into();
            }

            WRITE_FAN_OSCILLATION => {
                let oscillation: FanOscillation = borsh::from_slice(&payload)?;
                req.has_swing_mode = true;
                req.swing_mode = fan_oscillation_to_swing(&oscillation).into();
            }

            WRITE_REAL => {
                let temperature: Real = borsh::from_slice(&payload)?;
                req.has_target_temperature = true;
                req.target_temperature = temperature as f32;
            }

            WRITE_TEXT => {
                let text: Text = borsh::from_slice(&payload)?;
                req.has_custom_preset = true;
                req.custom_preset = text;
            }

            DESELECT_ENTITY | END_TRANSACTION => {
                unreachable!();
            }

            _ => {
                println!("Climate got unexpected command {cmd_id} during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::ClimateCommandRequest, &req)
        .await
}
