use std::sync::Arc;

use serde::Serialize;

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<i32> for EntityCommand {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<IntState> for EntityState {
    fn from(value: IntState) -> Self {
        Self::Int(value)
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct IntState {
    value: i32,
}

pub fn dispatch(
    cmd: i32,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<i32> for IntState {
    fn from(value: i32) -> Self {
        Self { value }
    }
}

impl From<IntState> for i32 {
    fn from(value: IntState) -> Self {
        value.value
    }
}

impl From<&IntState> for i32 {
    fn from(value: &IntState) -> Self {
        value.value
    }
}

impl IntState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (0, true, false);
        for state in states {
            if let EntityState::Int(state) = state {
                let state: i32 = state.into();
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
                value: EntityState::Int(last_state.into()),
                homogeneous,
            }),
        }
    }
}
