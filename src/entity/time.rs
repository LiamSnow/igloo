use std::{error::Error, sync::Arc};

use chrono::NaiveTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<NaiveTime> for EntityCommand {
    fn from(value: NaiveTime) -> Self {
        Self::Time(value)
    }
}

impl From<TimeState> for EntityState {
    fn from(value: TimeState) -> Self {
        Self::Time(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TimeState {
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    value: NaiveTime,
}

pub fn dispatch(
    cmd: NaiveTime,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<NaiveTime> for TimeState {
    fn from(value: NaiveTime) -> Self {
        Self { value }
    }
}

impl TimeState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (NaiveTime::default(), true, false);
        for state in states {
            if let EntityState::Time(state) = state {
                let state = state.value.clone();
                if homogeneous {
                    if first {
                        first = false;
                    } else {
                        homogeneous = state == last_state;
                    }
                    last_state = state;
                }
            }
        }
        match first {
            true => None,
            false => Some(AveragedEntityState {
                value: EntityState::Time(last_state.into()),
                homogeneous,
            }),
        }
    }
}


pub fn deserialize_time<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveTime, D::Error> {
    let time_str = String::deserialize(deserializer)?;
    NaiveTime::parse_from_str(&time_str, "%I:%M %p")
        .or_else(|_| NaiveTime::parse_from_str(&time_str, "%H:%M"))
        .map_err(serde::de::Error::custom)
}

pub fn parse_time(time_str: &str) -> Result<NaiveTime, Box<dyn Error>> {
    Ok(NaiveTime::parse_from_str(&time_str, "%I:%M %p")
        .or_else(|_| NaiveTime::parse_from_str(&time_str, "%H:%M"))?)
}

pub fn serialize_time<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&time.format("%I:%M %p").to_string())
}
