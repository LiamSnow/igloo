use std::sync::Arc;

use jiff::civil::Time;
use serde::{Deserialize, Serialize};

use crate::{cli::error::DispatchError, device::ids::DeviceIDSelection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<Time> for EntityCommand {
    fn from(value: Time) -> Self {
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
    value: Time,
}

pub fn dispatch(
    cmd: Time,
    sel_str: String,
    sel: DeviceIDSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<Time> for TimeState {
    fn from(value: Time) -> Self {
        Self { value }
    }
}

impl TimeState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (Time::default(), true, false);
        for state in states {
            if let EntityState::Time(state) = state {
                let state = state.value.clone();
                if first {
                    first = false;
                }
                if homogeneous && !first {
                    homogeneous = state == last_state;
                }
                last_state = state;
            }
        }
        match first {
            true => None,
            false => Some(AveragedEntityState {
                value: EntityState::Time(last_state.into()),
                homogeneous,
                disconnection_stats: None
            }),
        }
    }
}
