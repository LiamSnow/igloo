use std::sync::Arc;

use jiff::civil::{Time, Weekday};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

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

impl Default for Weekly {
    fn default() -> Self {
        Self::work_days(Time::default())
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
            time
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
                Weekday::Saturday
            ],
            time
        }
    }
}

impl From<Weekly> for EntityCommand {
    fn from(value: Weekly) -> Self {
        Self::Weekly(value)
    }
}

impl From<WeeklyState> for EntityState {
    fn from(value: WeeklyState) -> Self {
        Self::Weekly(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WeeklyState {
    value: Weekly,
}

pub fn dispatch(
    cmd: Weekly,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<Weekly> for WeeklyState {
    fn from(value: Weekly) -> Self {
        Self { value }
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
                }
                else {
                    first_state = Some(state.clone());
                }
            }
        }
        match first_state {
            Some(value) => Some(AveragedEntityState {
                value: EntityState::Weekly(value),
                homogeneous
            }),
            None => None,
        }
    }
}

pub fn deserialize_weekdays<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Weekday>, D::Error> {
    let strs = Vec::<String>::deserialize(deserializer)?;

    let mut weekdays = Vec::with_capacity(strs.len());
    for s in strs {
        let weekday = match s.to_lowercase().as_str() {
            "monday" => Weekday::Monday,
            "tuesday" => Weekday::Tuesday,
            "wednesday" => Weekday::Wednesday,
            "thursday" => Weekday::Thursday,
            "friday" => Weekday::Friday,
            "saturday" => Weekday::Saturday,
            "sunday" => Weekday::Sunday,
            _ => return Err(serde::de::Error::custom(format!("Unknown weekday: {}", s))),
        };
        weekdays.push(weekday);
    }

    Ok(weekdays)
}

pub fn serialize_weekdays<S: Serializer>(weekdays: &Vec<Weekday>, serializer: S) -> Result<S::Ok, S::Error> {
    let v: Vec<String> = weekdays.iter().map(|weekday| {
        match weekday {
            Weekday::Monday => "Monday".to_string(),
            Weekday::Tuesday => "Tuesday".to_string(),
            Weekday::Wednesday => "Wednesday".to_string(),
            Weekday::Thursday => "Thursday".to_string(),
            Weekday::Friday => "Friday".to_string(),
            Weekday::Saturday => "Saturday".to_string(),
            Weekday::Sunday => "Sunday".to_string(),
        }
    }).collect();

    v.serialize(serializer)
}
