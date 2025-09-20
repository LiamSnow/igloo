use serde::{Deserialize, Serialize};

use crate::{Components, types::Type};

#[derive(Debug, Serialize, Deserialize)]
pub enum Standard {
    Light,
    FloatSensor,
    IntSensor,
    LongSensor,
    ClimateMode,
    Temperature,
    Fan,
    Lock,
    MediaPlayer,
    Switch,
    Cover,
    BinarySensor,
    Button,
    Number,
    Alarm,
    Thermostat,
    Camera,
    Vacuum,
    Select,
    Text,
}

impl Components {
    pub fn conforms(&self, std: Standard) -> bool {
        use Type::*;
        match std {
            Standard::Light => {
                self.has_req_comp_of("on", Bool)
                    && self.has_opt_comp_of("brightness", Float)
                    && self.has_opt_comp_of("color_temperature", Float)
                    && self.has_opt_comp_of("color", Color)
            }
            Standard::FloatSensor => {
                self.has_req_comp_of("unit", String) && self.has_req_comp_of("value", Float)
            }
            Standard::IntSensor => {
                self.has_req_comp_of("unit", String) && self.has_req_comp_of("value", Int)
            }
            Standard::LongSensor => {
                self.has_req_comp_of("unit", String) && self.has_req_comp_of("value", Long)
            }
            Standard::ClimateMode => self.has_req_string_of(
                "climate_mode",
                vec![
                    "off",
                    "auto",
                    "heat",
                    "cool",
                    "heat_cool",
                    "fan_only",
                    "dry",
                    "eco",
                ],
            ),
            Standard::Temperature => self.has_req_comp_of("temperature", Float),
            Standard::Fan => {
                (self.has_req_comp_of("fan_speed", Float)
                    || self.has_req_string_of("fan_speed", vec!["auto"]))
                    && self.has_opt_string_of("fan_direction", vec!["forward", "reverse"])
                    && self.has_opt_string_of(
                        "fan_oscillation",
                        vec!["off", "vertical", "horizontal", "both"],
                    )
            }
            Standard::Lock => self.has_req_string_of(
                "lock_state",
                vec![
                    "unknown",
                    "locked",
                    "unlocked",
                    "jammed",
                    "locking",
                    "unlocking",
                ],
            ),
            Standard::MediaPlayer => {
                self.has_req_string_of("media_state", vec!["unknown", "idle", "playing", "paused"])
            }
            Standard::Switch => self.has_req_comp_of("on", Bool),
            Standard::Cover => {
                self.has_req_comp_of("position", Float)
                    && self.has_opt_string_of(
                        "state",
                        vec!["opening", "closing", "stopped", "open", "closed"],
                    )
                    && self.has_opt_comp_of("tilt", Float)
            }
            Standard::BinarySensor => {
                self.has_req_comp_of("state", Bool) && self.has_opt_comp_of("sensor_type", String)
            }
            Standard::Button => {
                self.has_req_string_of(
                    "action",
                    vec!["single", "double", "long", "release", "pressed"],
                ) || self.has_req_comp_of("pressed", Bool)
            }
            Standard::Number => {
                self.has_req_comp_of("value", Float)
                    && self.has_opt_comp_of("min", Float)
                    && self.has_opt_comp_of("max", Float)
                    && self.has_opt_comp_of("step", Float)
                    && self.has_opt_comp_of("unit", String)
            }
            Standard::Alarm => {
                self.has_req_string_of(
                    "arm_state",
                    vec![
                        "disarmed",
                        "armed_home",
                        "armed_away",
                        "armed_night",
                        "armed_vacation",
                        "armed_custom",
                        "pending",
                        "triggered",
                        "arming",
                        "disarming",
                    ],
                ) && self.has_opt_comp_of("code", String)
            }
            Standard::Thermostat => {
                self.has_req_comp_of("current_temperature", Float)
                    && self.has_req_comp_of("target_temperature", Float)
                    && self.conforms(Standard::ClimateMode)
                    && self.has_opt_comp_of("target_temperature_high", Float)
                    && self.has_opt_comp_of("target_temperature_low", Float)
                    && self.has_opt_comp_of("current_humidity", Float)
            }
            Standard::Camera => {
                self.has_req_comp_of("streaming", Bool)
                    && self.has_opt_comp_of("recording", Bool)
                    && self.has_opt_comp_of("motion_detected", Bool)
                    && self.has_opt_comp_of("snapshot_url", String)
                    && self.has_opt_comp_of("stream_url", String)
            }
            Standard::Vacuum => {
                self.has_req_string_of(
                    "state",
                    vec!["docked", "idle", "cleaning", "returning", "paused", "error"],
                ) && self.has_opt_comp_of("battery_level", Float)
                    && self.has_opt_comp_of("fan_speed", Float)
                    && self.has_opt_string_of(
                        "fan_speed",
                        vec!["auto", "silent", "standard", "medium", "high", "max"],
                    )
            }
            Standard::Select => {
                self.has_req_comp_of("selected", String) && self.has_req_list_of("options", String)
            }
            Standard::Text => {
                self.has_req_comp_of("text", String)
                    && self.has_opt_comp_of("max_length", Int)
                    && self.has_opt_comp_of("pattern", String)
            }
        }
    }
}
