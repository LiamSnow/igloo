use crate::Color;
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

#[derive(Debug, Clone, PartialEq, Eq, Copy, BorshSerialize, BorshDeserialize)]
pub enum PinType {
    /// execution flow
    Flow,
    /// holds value
    Value(PenguinType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, BorshSerialize, BorshDeserialize)]
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

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum PenguinValue {
    Int(i64),
    Real(f64),
    Text(String),
    Bool(bool),
    Color(Color),
}

impl PinType {
    pub fn can_cast(self, to: Self) -> bool {
        match (self, to) {
            (PinType::Value(from), PinType::Value(to)) => from.can_cast(to),
            _ => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        match (self, to) {
            (PinType::Value(from), PinType::Value(to)) => from.cast_name(to),
            _ => None,
        }
    }

    pub fn can_connect_to(&self, target: PinType) -> bool {
        *self == target || self.can_cast(target)
    }

    pub fn stroke(&self) -> &str {
        match self {
            PinType::Flow => "white",
            PinType::Value(vt) => vt.color(),
        }
    }

    pub fn stroke_width(&self) -> u8 {
        match self {
            PinType::Flow => 4,
            PinType::Value(_) => 2,
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

impl Color {
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        Some(Color {
            r: u8::from_str_radix(&s[0..2], 16).ok()?,
            g: u8::from_str_radix(&s[2..4], 16).ok()?,
            b: u8::from_str_radix(&s[4..6], 16).ok()?,
        })
    }
}
