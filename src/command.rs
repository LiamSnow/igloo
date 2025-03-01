use serde::Serialize;

use crate::cli::model::{LightAction, SwitchState};

#[derive(Debug, Clone)]
pub enum SubdeviceCommand {
    Light(LightAction),
    Switch(SwitchState),
}

pub struct TargetedSubdeviceCommand {
    /// if None -> apply to all applicable subdevices
    pub subdev_name: Option<String>,
    pub cmd: SubdeviceCommand,
}

#[derive(Debug, Clone, Serialize)]
pub enum SubdeviceState {
    Light(LightState),
    Switch(SwitchState),
}

#[derive(Serialize, Clone)]
pub struct AveragedSubdeviceState {
    pub value: SubdeviceState,
    pub homogeneous: bool,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum SubdeviceType {
    Light,
    Switch,
}

impl SubdeviceState {
    pub fn get_type(&self) -> SubdeviceType {
        match self {
            Self::Light(..) => SubdeviceType::Light,
            Self::Switch(..) => SubdeviceType::Switch,
        }
    }
}

impl From<LightAction> for SubdeviceCommand {
    fn from(value: LightAction) -> Self {
        Self::Light(value)
    }
}

impl From<SwitchState> for SubdeviceCommand {
    fn from(value: SwitchState) -> Self {
        Self::Switch(value)
    }
}

impl From<LightState> for SubdeviceState {
    fn from(value: LightState) -> Self {
        Self::Light(value)
    }
}

impl From<SwitchState> for SubdeviceState {
    fn from(value: SwitchState) -> Self {
        Self::Switch(value)
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct LightState {
    pub on: bool,
    pub color_on: bool,
    pub hue: Option<u16>,
    pub temp: Option<u32>,
    pub brightness: Option<u8>,
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

        // Find max and min components
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);

        // Check if the color is grayscale
        if (max - min).abs() < f32::EPSILON {
            return 0; // Arbitrary choice for grayscale
        }

        // Calculate hue based on which component is max
        let hue = if (r - max).abs() < f32::EPSILON {
            // Red is max
            if (b - min).abs() < f32::EPSILON {
                // Blue is min, Green is varying
                // h = 0-60
                60.0 * g
            } else {
                // Green is min, Blue is varying
                // h = 300-360
                300.0 + 60.0 * (1.0 - b)
            }
        } else if (g - max).abs() < f32::EPSILON {
            // Green is max
            if (r - min).abs() < f32::EPSILON {
                // Red is min, Blue is varying
                // h = 120-180
                120.0 + 60.0 * b
            } else {
                // Blue is min, Red is varying
                // h = 60-120
                60.0 + 60.0 * (1.0 - r)
            }
        } else { // Blue is max
            // Blue is max
            if (g - min).abs() < f32::EPSILON {
                // Green is min, Red is varying
                // h = 240-300
                240.0 + 60.0 * r
            } else {
                // Red is min, Green is varying
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
    pub fn avg(states: &Vec<Vec<Option<Self>>>) -> Option<AveragedSubdeviceState> {
        let (mut total, mut on_sum, mut color_on_sum) = (0, 0, 0);
        let (mut total_hue, mut hue_sum) = (0, 0);
        let (mut total_temp, mut temp_sum) = (0, 0);
        let (mut total_bright, mut bright_sum) = (0, 0);

        let mut last_state: &Self = &Default::default();
        let mut first = true;
        let mut homogeneous = true;

        for state in states.iter().flatten().filter_map(|s| s.as_ref()) {
            total += 1;
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

            if homogeneous {
                if first {
                    first = false;
                } else {
                    homogeneous = last_state.visibly_equal(state);
                }
                last_state = state;
            }
        }

        if total == 0 {
            return None;
        }

        Some(AveragedSubdeviceState {
            value: SubdeviceState::Light(Self {
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

impl SwitchState {
    pub fn avg(states: &Vec<Vec<Option<bool>>>) -> Option<AveragedSubdeviceState> {
        let (mut last_state, mut first, mut homogeneous) = (false, true, false);
        for state in states.iter().flatten().filter_map(|s| s.as_ref()) {
            if homogeneous {
                if first {
                    first = false;
                } else {
                    homogeneous = *state == last_state;
                }
                last_state = *state;
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
