use std::sync::Arc;

use clap_derive::ValueEnum;
use serde::Serialize;

use crate::{cli::error::DispatchError, elements::AveragedSubdeviceState, selector::Selection, state::IglooState};

use super::{SubdeviceCommand, SubdeviceState};

impl From<SwitchState> for SubdeviceCommand {
    fn from(value: SwitchState) -> Self {
        Self::Switch(value)
    }
}

impl From<SwitchState> for SubdeviceState {
    fn from(value: SwitchState) -> Self {
        Self::Switch(value)
    }
}

pub type SwitchCommand = SwitchState;

#[derive(ValueEnum, Clone, Debug, Serialize)]
pub enum SwitchState {
    On,
    Off,
}

impl SwitchState {
    pub fn dispatch(
        self,
        target: String,
        sel: Selection,
        state: &Arc<IglooState>,
    ) -> Result<Option<String>, DispatchError> {
        sel.execute(&state, SubdeviceCommand::Switch(self))
            .map_err(|e| DispatchError::DeviceChannelErorr(target, e))?;
        Ok(None)
    }
}

impl Default for SwitchState {
    fn default() -> Self {
        Self::Off
    }
}

impl From<bool> for SwitchState {
    fn from(value: bool) -> Self {
        match value {
            true => SwitchState::On,
            false => SwitchState::Off,
        }
    }
}

impl From<SwitchState> for bool {
    fn from(value: SwitchState) -> Self {
        match value {
            SwitchState::On => true,
            SwitchState::Off => false,
        }
    }
}

impl From<&SwitchState> for bool {
    fn from(value: &SwitchState) -> Self {
        match value {
            SwitchState::On => true,
            SwitchState::Off => false,
        }
    }
}

impl SwitchState {
    pub fn avg(states: Vec<&SubdeviceState>) -> Option<AveragedSubdeviceState> {
        let (mut last_state, mut first, mut homogeneous) = (false, true, false);
        for state in states {
            if let SubdeviceState::Switch(state) = state {
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
            false => Some(AveragedSubdeviceState {
                value: SubdeviceState::Switch(last_state.into()),
                homogeneous
            }),
        }
    }
}

