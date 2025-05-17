use std::{str::FromStr, sync::Arc};

use clap_derive::Subcommand;
use jiff::civil::{Time, Weekday};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{cli::error::DispatchError, device::ids::DeviceSelection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Weekly {
    #[serde(
        deserialize_with = "deserialize_weekdays",
        serialize_with = "serialize_weekdays"
    )]
    pub days: Vec<Weekday>,
    pub time: Time,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WeekdayList(
    #[serde(
        deserialize_with = "deserialize_weekdays",
        serialize_with = "serialize_weekdays"
    )]
    pub Vec<Weekday>,
);

#[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
pub enum WeeklyCommand {
    #[command(alias = "days")]
    SetDays { days: WeekdayList },
    #[command(alias = "time")]
    SetTime { time: Time },
    #[command(alias = "all")]
    SetAll { days: WeekdayList, time: Time },
}

impl Default for Weekly {
    fn default() -> Self {
        Self::work_days(Time::default())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WeeklyState {
    value: Weekly,
}

pub fn dispatch(
    cmd: WeeklyCommand,
    sel_str: String,
    sel: DeviceSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<WeeklyCommand> for EntityCommand {
    fn from(value: WeeklyCommand) -> Self {
        Self::Weekly(value)
    }
}

impl From<Weekly> for WeeklyState {
    fn from(value: Weekly) -> Self {
        Self { value }
    }
}

impl From<WeeklyState> for EntityState {
    fn from(value: WeeklyState) -> Self {
        Self::Weekly(value)
    }
}

impl Weekly {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let mut homogeneous = true;
        let mut first_state = None;
        for state in states {
            if let EntityState::Weekly(state) = state {
                if first_state.is_some() {
                    homogeneous = false;
                } else {
                    first_state = Some(state.clone());
                }
            }
        }
        match first_state {
            Some(value) => Some(AveragedEntityState {
                value: EntityState::Weekly(value),
                homogeneous,
            }),
            None => None,
        }
    }
}

pub fn deserialize_weekdays<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Weekday>, D::Error> {
    str_to_weekdays(&String::deserialize(deserializer)?)
        .ok_or(serde::de::Error::custom(format!("invalid weekday")))
}

pub fn str_to_weekdays(str: &str) -> Option<Vec<Weekday>> {
    let strs: Vec<&str> = str.split(",").collect();
    let mut res = Vec::with_capacity(strs.len());

    for s in strs {
        let weekday = match s.to_lowercase().as_str() {
            "monday" | "mon" | "m" => Weekday::Monday,
            "tuesday" | "tue" | "tues" | "t" => Weekday::Tuesday,
            "wednesday" | "wed" | "w" => Weekday::Wednesday,
            "thursday" | "thurs" | "thu" | "r" => Weekday::Thursday,
            "friday" | "fri" | "f" => Weekday::Friday,
            "saturday" | "sat" => Weekday::Saturday,
            "sunday" | "sun" => Weekday::Sunday,
            _ => return None,
        };
        res.push(weekday);
    }

    Some(res)
}

pub fn serialize_weekdays<S: Serializer>(
    weekdays: &Vec<Weekday>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let v: Vec<String> = weekdays
        .iter()
        .map(|weekday| match weekday {
            Weekday::Monday => "Monday".to_string(),
            Weekday::Tuesday => "Tuesday".to_string(),
            Weekday::Wednesday => "Wednesday".to_string(),
            Weekday::Thursday => "Thursday".to_string(),
            Weekday::Friday => "Friday".to_string(),
            Weekday::Saturday => "Saturday".to_string(),
            Weekday::Sunday => "Sunday".to_string(),
        })
        .collect();

    v.serialize(serializer)
}

impl FromStr for WeekdayList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match str_to_weekdays(s) {
            Some(r) => Ok(WeekdayList(r)),
            None => Err("invalid weekday".to_string()),
        }
    }
}

impl Weekly {
    pub fn work_days(time: Time) -> Self {
        Self {
            days: vec![
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
            ],
            time,
        }
    }

    pub fn all_days(time: Time) -> Self {
        Self {
            days: vec![
                Weekday::Sunday,
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
                Weekday::Saturday,
            ],
            time,
        }
    }
}

impl Weekly {
    pub fn apply_cmd(&mut self, cmd: WeeklyCommand) {
        match cmd {
            WeeklyCommand::SetDays { days } => self.days = days.0,
            WeeklyCommand::SetTime { time } => self.time = time,
            WeeklyCommand::SetAll { days, time } => {
                self.days = days.0;
                self.time = time;
            }
        }
    }
}

impl WeeklyState {
    pub fn apply_cmd(&mut self, cmd: WeeklyCommand) {
        self.value.apply_cmd(cmd);
    }
}
