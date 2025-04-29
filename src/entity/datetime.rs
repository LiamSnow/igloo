use std::sync::Arc;

use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<NaiveDateTime> for EntityCommand {
    fn from(value: NaiveDateTime) -> Self {
        Self::DateTime(value)
    }
}

impl From<DateTimeState> for EntityState {
    fn from(value: DateTimeState) -> Self {
        Self::DateTime(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DateTimeState {
    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime"
    )]
    value: NaiveDateTime,
}

pub fn dispatch(
    cmd: NaiveDateTime,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<NaiveDateTime> for DateTimeState {
    fn from(value: NaiveDateTime) -> Self {
        Self { value }
    }
}

impl DateTimeState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (NaiveDateTime::default(), true, false);
        for state in states {
            if let EntityState::DateTime(state) = state {
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
                value: EntityState::DateTime(last_state.into()),
                homogeneous,
            }),
        }
    }
}


pub fn deserialize_datetime<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDateTime, D::Error> {
    let time_str = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&time_str, "%m/%d/%y %I:%M %p")
        .or_else(|_| NaiveDateTime::parse_from_str(&time_str, "%m/%d/%y %H:%M"))
        .map_err(serde::de::Error::custom)
}

pub fn serialize_datetime<S: Serializer>(datetime: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&datetime.format("%m/%d/%y %H:%M").to_string())
}
