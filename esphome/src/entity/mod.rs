use async_trait::async_trait;
use igloo_interface::{FloeWriterDefault, SensorStateClass, Unit};

use crate::api;

pub mod alarm_control_panel;
pub mod binary_sensor;
pub mod button;
pub mod camera;
pub mod climate;
pub mod cover;
pub mod date;
pub mod date_time;
pub mod event;
pub mod fan;
pub mod light;
pub mod lock;
pub mod media_player;
pub mod number;
pub mod select;
pub mod sensor;
pub mod siren;
pub mod switch;
pub mod text;
pub mod text_sensor;
pub mod time;
pub mod update;
pub mod valve;

#[async_trait]
pub trait EntityUpdate {
    fn key(&self) -> u32;
    fn should_skip(&self) -> bool {
        false
    }
    async fn write_to(&self, writer: &mut FloeWriterDefault) -> Result<(), std::io::Error>;
}

#[async_trait]
pub trait EntityRegister {
    async fn register(
        self,
        device: &mut crate::device::Device,
        writer: &mut FloeWriterDefault,
    ) -> Result<(), crate::device::DeviceError>;
}

pub async fn add_entity_category(
    writer: &mut FloeWriterDefault,
    category: api::EntityCategory,
) -> Result<(), std::io::Error> {
    match category {
        api::EntityCategory::None => {}
        api::EntityCategory::Config => {
            writer.config().await?;
        }
        api::EntityCategory::Diagnostic => {
            writer.diagnostic().await?;
        }
    }
    Ok(())
}

pub async fn add_icon(writer: &mut FloeWriterDefault, icon: &String) -> Result<(), std::io::Error> {
    if !icon.is_empty() {
        writer.icon(icon).await?;
    }
    Ok(())
}

pub async fn add_unit(
    writer: &mut FloeWriterDefault,
    unit_str: String,
) -> Result<(), std::io::Error> {
    // TODO log error, something else? if parsing failed..?
    if !unit_str.is_empty()
        && let Ok(unit) = Unit::try_from(unit_str)
    {
        writer.unit(&unit).await?;
    }
    Ok(())
}

pub async fn add_f32_bounds(
    writer: &mut FloeWriterDefault,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> Result<(), std::io::Error> {
    writer.float_min(&min).await?;
    writer.float_max(&max).await?;
    if let Some(step) = step {
        writer.float_step(&step).await?;
    }
    Ok(())
}

pub async fn add_device_class(
    writer: &mut FloeWriterDefault,
    device_class: String,
) -> Result<(), std::io::Error> {
    if !device_class.is_empty() {
        writer.device_class(&device_class).await?;
    }
    Ok(())
}

pub async fn add_number_mode(
    writer: &mut FloeWriterDefault,
    number_mode: api::NumberMode,
) -> Result<(), std::io::Error> {
    writer.number_mode(&number_mode.as_igloo()).await?;
    Ok(())
}

pub async fn add_climate_modes(
    writer: &mut FloeWriterDefault,
    modes: impl Iterator<Item = api::ClimateMode>,
) -> Result<(), std::io::Error> {
    let modes = modes.map(|m| m.as_igloo()).collect();
    writer.supported_climate_modes(&modes).await?;
    Ok(())
}

pub async fn add_fan_speeds(
    writer: &mut FloeWriterDefault,
    modes: impl Iterator<Item = api::ClimateFanMode>,
) -> Result<(), std::io::Error> {
    let speeds = modes.map(|m| m.as_igloo()).collect();
    writer.supported_fan_speeds(&speeds).await?;
    Ok(())
}

pub async fn add_fan_oscillations(
    writer: &mut FloeWriterDefault,
    modes: impl Iterator<Item = api::ClimateSwingMode>,
) -> Result<(), std::io::Error> {
    let modes = modes.map(|m| m.as_igloo()).collect();
    writer.supported_fan_oscillations(&modes).await?;
    Ok(())
}

pub async fn add_sensor_state_class(
    writer: &mut FloeWriterDefault,
    state_class: api::SensorStateClass,
) -> Result<(), std::io::Error> {
    match state_class {
        api::SensorStateClass::StateClassNone => {}
        api::SensorStateClass::StateClassMeasurement => {
            writer
                .sensor_state_class(&SensorStateClass::Measurement)
                .await?;
        }
        api::SensorStateClass::StateClassTotalIncreasing => {
            writer
                .sensor_state_class(&SensorStateClass::TotalIncreasing)
                .await?;
        }
        api::SensorStateClass::StateClassTotal => {
            writer.sensor_state_class(&SensorStateClass::Total).await?;
        }
    }
    Ok(())
}
