use std::error::Error;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum PenguinPinType {
    /// execution flow
    Flow,
    /// holds value
    Value(PenguinType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Serialize, Deserialize)]
pub enum PenguinType {
    #[display("Integer")]
    Int,
    #[display("Real")]
    Real,
    #[display("Text")]
    Text,
    #[display("Boolean")]
    Bool,
    #[display("Color")]
    Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
pub enum PenguinValue {
    #[display("{_0}")]
    Int(i64),
    #[display("{_0}")]
    Real(f64),
    #[display("{_0}")]
    Text(String),
    #[display("{_0}")]
    Bool(bool),
    #[display("{_0}")]
    Color(PenguinColor),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, Default)]
#[display("#{r:02x}{g:02x}{b:02x}")]
pub struct PenguinColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PenguinValue {
    pub fn default(r#type: &PenguinType) -> Self {
        match r#type {
            PenguinType::Int => PenguinValue::Int(0),
            PenguinType::Real => PenguinValue::Real(0.),
            PenguinType::Text => PenguinValue::Text(String::default()),
            PenguinType::Bool => PenguinValue::Bool(false),
            PenguinType::Color => PenguinValue::Color(PenguinColor::default()),
        }
    }

    pub fn set_from_string(&mut self, value: String) -> Result<(), Box<dyn Error>> {
        match self {
            PenguinValue::Int(v) => {
                *v = value.parse()?;
            }
            PenguinValue::Real(v) => {
                *v = value.parse()?;
            }
            PenguinValue::Text(v) => {
                *v = value;
            }
            PenguinValue::Bool(v) => {
                *v = value.parse()?;
            }
            PenguinValue::Color(v) => {
                *v = PenguinColor::from_hex(&value)
                    .ok_or("Invalid PenguinColor hex value".to_string())?;
            }
        }
        Ok(())
    }
}

impl PenguinPinType {
    pub fn can_cast(self, to: Self) -> bool {
        match (self, to) {
            (PenguinPinType::Value(from), PenguinPinType::Value(to)) => from.can_cast(to),
            _ => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        match (self, to) {
            (PenguinPinType::Value(from), PenguinPinType::Value(to)) => from.cast_name(to),
            _ => None,
        }
    }

    pub fn can_connect_to(&self, target: PenguinPinType) -> bool {
        *self == target || self.can_cast(target)
    }

    pub fn stroke(&self) -> &str {
        match self {
            PenguinPinType::Flow => "white",
            PenguinPinType::Value(vt) => vt.color(),
        }
    }

    pub fn stroke_width(&self) -> u8 {
        match self {
            PenguinPinType::Flow => 4,
            PenguinPinType::Value(_) => 2,
        }
    }
}

impl PenguinType {
    pub fn color(&self) -> &'static str {
        match self {
            PenguinType::Text => "#9b59b6",
            PenguinType::Bool => "#e74c3c",
            PenguinType::Int => "#3498db",
            PenguinType::Real => "#27ae60",
            PenguinType::Color => "#f39c12",
        }
    }

    pub fn can_cast(self, to: Self) -> bool {
        match to {
            PenguinType::Int => self != PenguinType::Color,
            PenguinType::Real => self != PenguinType::Color,
            PenguinType::Text => true,
            PenguinType::Bool => self != PenguinType::Color,
            PenguinType::Color => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        if !self.can_cast(to) {
            return None;
        }
        Some(format!(
            "cast_{}_to_{}",
            self.to_string().to_lowercase(),
            to.to_string().to_lowercase()
        ))
    }
}

impl PenguinColor {
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        Some(Self {
            r: u8::from_str_radix(&s[0..2], 16).ok()?,
            g: u8::from_str_radix(&s[2..4], 16).ok()?,
            b: u8::from_str_radix(&s[4..6], 16).ok()?,
        })
    }
}
