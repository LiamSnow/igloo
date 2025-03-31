use std::sync::Arc;

use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState, time::{deserialize_time, serialize_time}};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Weekly {
    pub days: Vec<Weekday>,
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub time: NaiveTime,
}

impl Default for Weekly {
    fn default() -> Self {
        Self::work_days(NaiveTime::default())
    }
}

impl Weekly {
    pub fn work_days(time: NaiveTime) -> Self {
        Self {
            days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            time
        }
    }

    pub fn all_days(time: NaiveTime) -> Self {
        Self {
            days: vec![
                Weekday::Sun,
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat
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
    //TODO serialize+de
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
