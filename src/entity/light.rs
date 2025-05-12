use std::sync::Arc;

use clap_derive::Subcommand;
use serde::Serialize;

use crate::{cli::error::DispatchError, selector::Selection, state::IglooState};

use super::{EntityCommand, EntityState, AveragedEntityState};

#[derive(Subcommand, Debug, Clone)]
pub enum LightCommand {
    /// Turn the light on
    On,
    /// Turn the light off
    Off,
    /// Set the light color using an hue value
    #[command(alias = "hue")]
    Color { hue: Option<u16> },
    /// Set the light temperature
    #[command(alias = "temp")]
    Temperature { temp: Option<u32> },
    /// Set the light brightness
    #[command(alias = "bri")]
    Brightness { brightness: u8 },
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct LightState {
    pub on: bool,
    pub color_on: bool,
    pub hue: Option<u16>,
    pub temp: Option<u32>,
    pub brightness: Option<u8>,
}

pub fn dispatch(
    cmd: LightCommand,
    sel_str: String,
    sel: Selection,
    state: &Arc<IglooState>,
) -> Result<Option<String>, DispatchError> {
    sel.execute(&state, cmd.into())
        .map_err(|e| DispatchError::DeviceChannelError(sel_str, e))?;
    Ok(None)
}

impl From<LightCommand> for EntityCommand {
    fn from(value: LightCommand) -> Self {
        Self::Light(value)
    }
}

impl From<LightState> for EntityState {
    fn from(value: LightState) -> Self {
        Self::Light(value)
    }
}

//Values as percentages (IE 0.-1.)
pub struct RGBF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl RGBF32 {
    /// Basically HSL to RGB with S=100%, L=50%
    pub fn from_hue(hue: u16) -> Self {
        let h = hue % 360;
        let (r, g, b) = match h {
            0..=59 => {
                let f = h as f32 / 60.0;
                (1., f, 0.)
            },
            60..=119 => {
                let f = (h - 60) as f32 / 60.0;
                (1. - f, 1., 0.)
            },
            120..=179 => {
                let f = (h - 120) as f32 / 60.0;
                (0., 1., f)
            },
            180..=239 => {
                let f = (h - 180) as f32 / 60.0;
                (0., 1. - f, 1.)
            },
            240..=299 => {
                let f = (h - 240) as f32 / 60.0;
                (f, 0., 1.)
            },
            _ => {
                let f = (h - 300) as f32 / 60.0;
                (1., 0., 1. - f)
            }
        };
        Self { r, g, b }
    }

    pub fn to_hue(&self) -> u16 {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);

        // grayscale
        if (max - min).abs() < f32::EPSILON {
            return 0;
        }

        let hue = if (r - max).abs() < f32::EPSILON {
            // Red max
            if (b - min).abs() < f32::EPSILON {
                // Blue min, Green varying
                // h = 0-60
                60.0 * g
            } else {
                // Green min, Blue varying
                // h = 300-360
                300.0 + 60.0 * (1.0 - b)
            }
        } else if (g - max).abs() < f32::EPSILON {
            // Green max
            if (r - min).abs() < f32::EPSILON {
                // Red min, Blue varying
                // h = 120-180
                120.0 + 60.0 * b
            } else {
                // Blue min, Red varying
                // h = 60-120
                60.0 + 60.0 * (1.0 - r)
            }
        } else {
            // Blue max
            if (g - min).abs() < f32::EPSILON {
                // Green min, Red varying
                // h = 240-300
                240.0 + 60.0 * r
            } else {
                // Red min, Green varying
                // h = 180-240
                180.0 + 60.0 * (1.0 - g)
            }
        };

        (hue.round() as u16) % 360
    }
}

impl LightState {
    /// Average a set of colors
    /// Only avgs Some colors, if there are no colors it returns None (same for temp and bri)
    pub fn avg(states: Vec<&EntityState>) -> Option<AveragedEntityState> {
        let total = states.len();
        let (mut on_sum, mut color_on_sum) = (0, 0);
        let (mut total_hue, mut hue_sum) = (0, 0);
        let (mut total_temp, mut temp_sum) = (0, 0);
        let (mut total_bright, mut bright_sum) = (0, 0);

        let mut last_state: &Self = &Default::default();
        let mut first = true;
        let mut homogeneous = true;

        for state in states {
            if let EntityState::Light(state) = state {
                on_sum += state.on as u32;
                color_on_sum += state.color_on as u32;
                if let Some(hue) = state.hue {
                    total_hue += 1;
                    hue_sum += hue as u32;
                }
                if let Some(temp) = state.temp {
                    total_temp += 1;
                    temp_sum += temp;
                }
                if let Some(bright) = state.brightness {
                    total_bright += 1;
                    bright_sum += bright as u32;
                }

                if first {
                    first = false;
                }

                if homogeneous && !first {
                    homogeneous = last_state.visibly_equal(state);
                }
                last_state = state;
            }
        }

        if total == 0 {
            return None;
        }

        Some(AveragedEntityState {
            value: EntityState::Light(Self {
                on: (on_sum as f32 / total as f32) >= 0.5,
                color_on: (color_on_sum as f32 / total as f32) >= 0.5,
                hue: if total_hue > 0 {
                    Some((hue_sum as f32 / total_temp as f32) as u16)
                } else {
                    None
                },
                temp: if total_temp > 0 {
                    Some((temp_sum as f32 / total_temp as f32) as u32)
                } else {
                    None
                },
                brightness: if total_bright > 0 {
                    Some((bright_sum as f32 / total_temp as f32) as u8)
                } else {
                    None
                },
            }),
            homogeneous,
        })
    }

    /// Returns if the lights are visibly equal
    /// For example: both lights are off, but have different color values
    pub fn visibly_equal(&self, other: &Self) -> bool {
        if self.on != other.on || self.color_on != other.color_on {
            return false;
        }
        if !self.on {
            return true;
        }

        match (self.brightness, other.brightness) {
            (Some(a), Some(b)) => {
                if a != b {
                    return false;
                }
            }
            _ => {}
        }

        if self.color_on {
            return self.hue == other.hue;
        }

        match (self.temp, other.temp) {
            (Some(a), Some(b)) => {
                if a != b {
                    return false;
                }
            }
            _ => {}
        }

        true
    }
}
