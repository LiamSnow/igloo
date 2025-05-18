use std::sync::Arc;

use serde::Serialize;

use crate::{cli::error::DispatchError, device::ids::DeviceIDSelection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<f32> for EntityCommand {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<FloatState> for EntityState {
    fn from(value: FloatState) -> Self {
        Self::Float(value)
    }
}

#[derive(Clone, Debug, Serialize, Default)]
#[serde(transparent)]
pub struct FloatState {
    value: f32,
}

pub fn dispatch(
    cmd: f32,
    sel_str: String,
    sel: DeviceIDSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<f32> for FloatState {
    fn from(value: f32) -> Self {
        Self { value }
    }
}

impl From<FloatState> for f32 {
    fn from(value: FloatState) -> Self {
        value.value
    }
}

impl From<&FloatState> for f32 {
    fn from(value: &FloatState) -> Self {
        value.value
    }
}

impl FloatState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (0., true, false);
        for state in states {
            if let EntityState::Float(state) = state {
                let state: f32 = state.into();
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
                value: EntityState::Float(last_state.into()),
                homogeneous,
                disconnection_stats: None
            }),
        }
    }
}
