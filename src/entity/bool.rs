use std::sync::Arc;

use clap_derive::{Subcommand, ValueEnum};
use serde::Serialize;

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{EntityCommand, EntityState, AveragedEntityState};

impl From<BoolCommand> for EntityCommand {
    fn from(value: BoolState) -> Self {
        Self::Bool(value)
    }
}

impl From<BoolState> for EntityState {
    fn from(value: BoolState) -> Self {
        Self::Bool(value)
    }
}

pub type BoolCommand = BoolState;

#[derive(ValueEnum, Clone, Debug, Serialize, Subcommand)]
pub enum BoolState {
    // #[command(alias = "on")]
    True,
    // #[command(alias = "off")]
    False,
}

pub fn dispatch(
    value: BoolCommand,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, value.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl Default for BoolState {
    fn default() -> Self {
        Self::False
    }
}

impl From<bool> for BoolState {
    fn from(value: bool) -> Self {
        match value {
            true => BoolState::True,
            false => BoolState::False,
        }
    }
}

impl From<BoolState> for bool {
    fn from(value: BoolState) -> Self {
        match value {
            BoolState::True => true,
            BoolState::False => false,
        }
    }
}

impl From<&BoolState> for bool {
    fn from(value: &BoolState) -> Self {
        match value {
            BoolState::True => true,
            BoolState::False => false,
        }
    }
}

impl BoolState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let (mut last_state, mut first, mut homogeneous) = (false, true, false);
        for state in states {
            if let EntityState::Bool(state) = state {
                let state: bool = state.into();
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
                value: EntityState::Bool(last_state.into()),
                homogeneous
            }),
        }
    }
}

