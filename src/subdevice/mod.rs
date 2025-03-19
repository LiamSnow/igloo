use chrono::NaiveTime;
use light::{LightCommand, LightState};
use serde::{Deserialize, Serialize, Serializer};
use switch::{SwitchCommand, SwitchState};

use crate::elements::AveragedSubdeviceState;

pub mod light;
pub mod switch;

#[derive(Debug, Clone)]
pub enum SubdeviceCommand {
    Light(LightCommand),
    Switch(SwitchCommand),
    Time(NaiveTime),
    Int(i32)
}

pub struct TargetedSubdeviceCommand {
    /// if None -> apply to all applicable subdevices
    pub subdev_name: Option<String>,
    pub cmd: SubdeviceCommand,
}

#[derive(Debug, Clone, Serialize)]
pub enum SubdeviceState {
    Light(LightState),
    Switch(SwitchState),
    #[serde(serialize_with = "serialize_time")]
    Time(NaiveTime),
    Int(i32)
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize, Serialize)]
pub enum SubdeviceType {
    Light,
    Switch,
    Time,
    Int
}

impl SubdeviceType {
    pub fn avg(&self, states: Vec<&SubdeviceState>) -> Option<AveragedSubdeviceState> {
        match self {
            SubdeviceType::Light => LightState::avg(states),
            SubdeviceType::Switch => SwitchState::avg(states),
            SubdeviceType::Time => todo!(),
            SubdeviceType::Int => todo!(),
        }
    }
}

impl SubdeviceState {
    pub fn get_type(&self) -> SubdeviceType {
        match self {
            Self::Light(..) => SubdeviceType::Light,
            Self::Switch(..) => SubdeviceType::Switch,
            Self::Time(..) => SubdeviceType::Time,
            Self::Int(..) => SubdeviceType::Int,
        }
    }
}

pub fn serialize_time<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&time.format("%H:%M").to_string())
}

pub fn parse_time(time_str: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(&time_str, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(&time_str, "%I:%M %p"))
}
