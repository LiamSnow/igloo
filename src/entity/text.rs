use std::sync::Arc;

use serde::Serialize;

use crate::{cli::error::DispatchError, device::ids::DeviceSelection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

impl From<String> for EntityCommand {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<TextState> for EntityState {
    fn from(value: TextState) -> Self {
        Self::Text(value)
    }
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct TextState {
    value: String,
}

pub fn dispatch(
    cmd: String,
    sel_str: String,
    sel: DeviceSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<String> for TextState {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl TextState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = ("".to_string(), true, false);
        for state in states {
            if let EntityState::Text(state) = state {
                let state = state.value.to_string();
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
                value: EntityState::Text(last_state.into()),
                homogeneous,
            }),
        }
    }
}
