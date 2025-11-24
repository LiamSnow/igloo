use igloo_interface::{Component, ClimateMode, FanOscillation, FanSpeed};
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

impl EntityRegister for api::ListEntitiesClimateResponse {
    fn comps(self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(5);
        add_entity_category(&mut comps, self.entity_category());
        add_icon(&mut comps, &self.icon);
        // add_f32_bounds(
        //     &mut comps,
        //     self.visual_min_temperature,
        //     self.visual_max_temperature,
        //     Some(self.visual_target_temperature_step),
        // );

        // add_climate_modes(&mut comps, self.supported_modes());

        // add_fan_speeds(&mut comps, self.supported_fan_modes());
        // add_fan_oscillations(&mut comps, self.supported_swing_modes());

        comps.push(Component::TextSelect);
        comps.push(Component::TextList(
            self
                .supported_presets()
                .map(|preset| format!("{preset:#?}"))
                .chain(self.supported_custom_presets.iter().cloned())
                .collect(),
        ));

        comps
    }
}

impl EntityUpdate for api::ClimateStateResponse {
    fn key(&self) -> u32 {
        self.key
    }

    fn comps(&self) -> Vec<Component> {
        let mut comps = Vec::with_capacity(6);
        comps.push(Component::Real(self.target_temperature as f64));
        comps.push(Component::ClimateMode(self.mode().as_igloo()));
        comps.push(Component::FanSpeed(self.fan_mode().as_igloo()));
        comps.push(Component::FanOscillation(self.swing_mode().as_igloo()));
        comps.push(Component::Text(format!("{:#?}", self.preset())));
        comps.push(Component::Text(self.custom_preset.clone()));
        comps
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
    comps: Vec<Component>,
) -> Result<(), DeviceError> {
    let mut req = api::ClimateCommandRequest {
        key,
        ..Default::default()
    };

    for comp in comps {
        use Component::*;
        match comp {
            ClimateMode(mode) => {
                req.has_mode = true;
                req.mode = climate_mode_to_api(&mode).into();
            }

            FanSpeed(speed) => {
                req.has_fan_mode = true;
                req.fan_mode = fan_speed_to_climate_fan(&speed).into();
            }

            FanOscillation(oscillation) => {
                req.has_swing_mode = true;
                req.swing_mode = fan_oscillation_to_swing(&oscillation).into();
            }

            Real(temperature) => {
                req.has_target_temperature = true;
                req.target_temperature = temperature as f32;
            }

            Text(text) => {
                req.has_custom_preset = true;
                req.custom_preset = text;
            }

            comp => {
                println!("Climate got unexpected component '{comp:?}' during transaction. Skipping..");
            }
        }
    }

    device
        .send_msg(MessageType::ClimateCommandRequest, &req)
        .await
}
