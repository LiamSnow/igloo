use std::sync::Arc;

use clap_derive::{Args, Subcommand};
use serde::Serialize;

use crate::{cli::error::DispatchError, device::ids::DeviceSelection, state::IglooState};

use super::{EntityCommand, EntityState, AveragedEntityState};

#[derive(Subcommand, Debug, Clone)]
pub enum FanCommand {
    SpeedAuto,
    Speed { percentage: u8 },
    Oscillation(FanOscillationArgs),
    Direction(FanDirectionArgs)
}

#[derive(Args, Debug, Clone)]
pub struct FanDirectionArgs {
    #[command(subcommand)]
    pub action: FanDirection,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum FanDirection {
    #[default]
    Forward,
    Reverse
}

#[derive(Args, Debug, Clone)]
pub struct FanOscillationArgs {
    #[command(subcommand)]
    pub action: FanOscillation,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub enum FanOscillation {
    #[default]
    Off,
    Vertical,
    Horizontal,
    Both
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct FanState {
    pub speed: Option<u8>,
    pub oscillation: FanOscillation,
    pub direction: FanDirection
}

impl From<u8> for FanDirection {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Reverse,
            _ => Self::Forward
        }
    }
}

impl From<u8> for FanOscillation {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Vertical,
            2 => Self::Horizontal,
            3 => Self::Both,
            _ => Self::Off
        }
    }
}

pub fn dispatch(
    cmd: FanCommand,
    sel_str: String,
    sel: DeviceSelection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<FanCommand> for EntityCommand {
    fn from(value: FanCommand) -> Self {
        Self::Fan(value)
    }
}

impl From<FanState> for EntityState {
    fn from(value: FanState) -> Self {
        Self::Fan(value)
    }
}

impl FanState {
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let total = states.len();
        let (mut total_speed, mut speed_sum) = (0, 0);
        let (mut oscillation_sum, mut direction_sum) = (0, 0);

        let mut last_state: &Self = &Default::default();
        let mut first = true;
        let mut homogeneous = true;

        for state in states {
            if let EntityState::Fan(state) = state {
                oscillation_sum += state.oscillation.clone() as u32;
                direction_sum += state.direction.clone() as u32;

                if let Some(speed) = state.speed {
                    total_speed += 1;
                    speed_sum += speed;
                }

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
            value: EntityState::Fan(Self {
                oscillation: ((oscillation_sum as f32 / total as f32) as u8).into(),
                direction: ((direction_sum as f32 / total as f32) as u8).into(),
                speed: if total_speed > 0 {
                    Some((speed_sum as f32 / total_speed as f32) as u8)
                } else {
                    None
                },
            }),
            homogeneous,
        })
    }
}
