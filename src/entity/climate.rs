use std::sync::Arc;

use clap_derive::{Args, Subcommand};
use serde::Serialize;

use crate::{cli::error::DispatchError, device::ids::DeviceIDSelection, state::IglooState};

use super::{AveragedEntityState, EntityCommand, EntityState};

#[derive(Subcommand, Debug, Clone)]
pub enum ClimateCommand {
    Mode(ClimateModeArgs),
    Temperature { value: i32 },
}

#[derive(Args, Debug, Clone)]
pub struct ClimateModeArgs {
    #[command(subcommand)]
    pub action: ClimateMode,
}

#[derive(Subcommand, Debug, Clone, Eq, PartialEq, Serialize, Default)]
pub enum ClimateMode {
    Off,
    #[default]
    Auto,
    Heat,
    Cool,
    HeatCool,
    Fan,
    Dry,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct ClimateState {
    temp: i32,
    mode: ClimateMode,
}

impl From<u8> for ClimateMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Off,
            2 => Self::Heat,
            3 => Self::Cool,
            4 => Self::HeatCool,
            5 => Self::Fan,
            6 => Self::Dry,
            _ => Self::Auto,
        }
    }
}

pub fn dispatch(
    cmd: ClimateCommand,
    sel_str: String,
    sel: DeviceIDSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<ClimateCommand> for EntityCommand {
    fn from(value: ClimateCommand) -> Self {
        Self::Climate(value)
    }
}

impl From<ClimateState> for EntityState {
    fn from(value: ClimateState) -> Self {
        Self::Climate(value)
    }
}

impl ClimateState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let total = states.len();
        let (mut mode_sum, mut temp_sum) = (0, 0);

        let mut last_state: &Self = &Default::default();
        let mut first = true;
        let mut homogeneous = true;

        for state in states {
            if let EntityState::Climate(state) = state {
                mode_sum += state.mode.clone() as u32;
                temp_sum += state.temp.clone() as u32;

                if first {
                    first = false;
                }
                if homogeneous && !first {
                    homogeneous = state == last_state;
                }
                last_state = state;
            }
        }

        if total == 0 {
            return None;
        }

        Some(AveragedEntityState {
            value: EntityState::Climate(Self {
                mode: ((mode_sum as f32 / total as f32) as u8).into(),
                temp: (temp_sum as f32 / total as f32) as i32,
            }),
            homogeneous,
            disconnection_stats: None
        })
    }
}
