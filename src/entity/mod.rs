use chrono::NaiveTime;
use light::{LightCommand, LightState};
use serde::{Deserialize, Serialize, Serializer};
use switch::{SwitchCommand, SwitchState};

pub mod light;
pub mod switch;

#[derive(Debug, Clone)]
pub enum EntityCommand {
    Light(LightCommand),
    Switch(SwitchCommand),
    Time(NaiveTime),
    Int(i32)
}

pub struct TargetedEntityCommand {
    /// if None -> apply to all applicable entities
    pub entity_name: Option<String>,
    pub cmd: EntityCommand,
}

#[derive(Debug, Clone, Serialize)]
pub enum EntityState {
    Light(LightState),
    Switch(SwitchState),
    #[serde(serialize_with = "serialize_time")]
    Time(NaiveTime),
    Int(i32)
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize, Serialize)]
pub enum EntityType {
    Light,
    Switch,
    Time,
    Int
}

#[derive(Serialize, Clone)]
pub struct AveragedEntityState {
    pub value: EntityState,
    pub homogeneous: bool,
}


impl EntityType {
    pub fn avg(&self, states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        match self {
            EntityType::Light => LightState::avg(states),
            EntityType::Switch => SwitchState::avg(states),
            EntityType::Time => todo!(),
            EntityType::Int => todo!(),
        }
    }
}

impl EntityState {
    pub fn get_type(&self) -> EntityType {
        match self {
            Self::Light(..) => EntityType::Light,
            Self::Switch(..) => EntityType::Switch,
            Self::Time(..) => EntityType::Time,
            Self::Int(..) => EntityType::Int,
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
