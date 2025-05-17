use std::sync::Arc;

use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

use crate::{cli::error::DispatchError, device::ids::DeviceSelection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<DateTime> for EntityCommand {
    fn from(value: DateTime) -> Self {
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
    value: DateTime,
}

pub fn dispatch(
    cmd: DateTime,
    sel_str: String,
    sel: DeviceSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<DateTime> for DateTimeState {
    fn from(value: DateTime) -> Self {
        Self { value }
    }
}

impl DateTimeState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (DateTime::default(), true, false);
        for state in states {
            if let EntityState::DateTime(state) = state {
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
                value: EntityState::DateTime(last_state.into()),
                homogeneous,
            }),
        }
    }
}
