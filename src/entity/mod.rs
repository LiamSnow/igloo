use chrono::{NaiveDateTime, NaiveTime};
use float::FloatState;
use int::IntState;
use light::{LightCommand, LightState};
use serde::{Deserialize, Serialize};
use bool::{BoolCommand, BoolState};
use text::TextState;
use time::TimeState;
use datetime::DateTimeState;
use weekly::{Weekly, WeeklyState};
use weekly_mult::{MultipleWeekly, MultipleWeeklyState};

pub mod light;
pub mod int;
pub mod float;
pub mod bool;
pub mod text;
pub mod time;
pub mod datetime;
pub mod weekly;
pub mod weekly_mult;

#[derive(Debug, Clone)]
pub enum EntityCommand {
    Light(LightCommand),
    Int(i32),
    Float(f32),
    Bool(BoolCommand),
    Text(String),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
    Weekly(Weekly),
    MultipleWeekly(MultipleWeekly)
}

pub struct TargetedEntityCommand {
    /// if None -> apply to all applicable entities
    pub entity_name: Option<String>,
    pub cmd: EntityCommand,
}

#[derive(Debug, Clone, Serialize)]
pub enum EntityState {
    Light(LightState),
    Int(IntState),
    Float(FloatState),
    Bool(BoolState),
    Text(TextState),
    Time(TimeState),
    DateTime(DateTimeState),
    Weekly(WeeklyState),
    MultipleWeekly(MultipleWeeklyState)
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Deserialize, Serialize)]
pub enum EntityType {
    Light,
    Int,
    Float,
    Bool,
    Text,
    Time,
    DateTime,
    Weekly,
    MultipleWeekly
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
            EntityType::Int => IntState::avg(states),
            EntityType::Float => FloatState::avg(states),
            EntityType::Bool => BoolState::avg(states),
            EntityType::Text => TextState::avg(states),
            EntityType::Time => TimeState::avg(states),
            EntityType::DateTime => DateTimeState::avg(states),
            EntityType::Weekly => Weekly::avg(states),
            EntityType::MultipleWeekly => MultipleWeekly::avg(states)
        }
    }
}

impl EntityState {
    pub fn get_type(&self) -> EntityType {
        match self {
            Self::Light(..) => EntityType::Light,
            Self::Int(..) => EntityType::Int,
            Self::Float(..) => EntityType::Float,
            Self::Bool(..) => EntityType::Bool,
            Self::Text(..) => EntityType::Text,
            Self::Time(..) => EntityType::Time,
            Self::DateTime(..) => EntityType::DateTime,
            Self::Weekly(..) => EntityType::Weekly,
            Self::MultipleWeekly(..) => EntityType::MultipleWeekly,
        }
    }
}
