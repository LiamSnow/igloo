use std::sync::Arc;

use chrono::{NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState, time::{deserialize_time, serialize_time}};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MultipleWeekly {
    pub days: Vec<Weekday>,
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub time: NaiveTime,
}


impl Default for MultipleWeekly {
    fn default() -> Self {
        Self {
            days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            time: NaiveTime::default()
        }
    }
}

impl From<MultipleWeekly> for EntityCommand {
    fn from(value: MultipleWeekly) -> Self {
        Self::MultipleWeekly(value)
    }
}

impl From<MultipleWeeklyState> for EntityState {
    fn from(value: MultipleWeeklyState) -> Self {
        Self::MultipleWeekly(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MultipleWeeklyState {
    //TODO serialize+de
    value: MultipleWeekly,
}

pub fn dispatch(
    cmd: MultipleWeekly,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<MultipleWeekly> for MultipleWeeklyState {
    fn from(value: MultipleWeekly) -> Self {
        Self { value }
    }
}

impl MultipleWeekly {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let mut homogeneous = true;
        let mut first_state = None;
        for state in states {
            if let EntityState::MultipleWeekly(state) = state {
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
                value: EntityState::MultipleWeekly(value),
                homogeneous
            }),
            None => None,
        }
    }
}
