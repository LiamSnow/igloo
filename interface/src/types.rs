use crate::{IglooEnumType, IglooEnumValue};
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;
#[cfg(feature = "penguin")]
use serde::{Deserialize, Serialize};

pub type IglooInteger = i64;
pub type IglooReal = f64;
pub type IglooText = String;
pub type IglooBoolean = bool;
pub type IglooIntegerList = Vec<i64>;
pub type IglooRealList = Vec<f64>;
pub type IglooTextList = Vec<String>;
pub type IglooBooleanList = Vec<bool>;
pub type IglooColorList = Vec<IglooColor>;
pub type IglooDateList = Vec<IglooDate>;
pub type IglooTimeList = Vec<IglooTime>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
pub enum IglooType {
    #[display("Integer")]
    Integer,
    #[display("Real")]
    Real,
    #[display("Text")]
    Text,
    #[display("Boolean")]
    Boolean,
    #[display("Color")]
    Color,
    #[display("Date")]
    Date,
    #[display("Time")]
    Time,

    #[display("IntegerList")]
    IntegerList,
    #[display("RealList")]
    RealList,
    #[display("TextList")]
    TextList,
    #[display("BooleanList")]
    BooleanList,
    #[display("ColorList")]
    ColorList,
    #[display("DateList")]
    DateList,
    #[display("TimeList")]
    TimeList,

    #[display("Enum")]
    Enum(IglooEnumType),
}

pub static IGLOO_PRIMITIVES: [IglooType; 14] = [
    IglooType::Integer,
    IglooType::Real,
    IglooType::Text,
    IglooType::Boolean,
    IglooType::Color,
    IglooType::Date,
    IglooType::Time,
    IglooType::IntegerList,
    IglooType::RealList,
    IglooType::TextList,
    IglooType::BooleanList,
    IglooType::ColorList,
    IglooType::DateList,
    IglooType::TimeList,
];

#[derive(Debug, Clone, PartialEq, Display)]
#[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
pub enum IglooValue {
    #[display("{_0}")]
    Integer(IglooInteger),
    #[display("{_0}")]
    Real(IglooReal),
    #[display("{_0}")]
    Text(IglooText),
    #[display("{_0}")]
    Boolean(IglooBoolean),
    #[display("{_0}")]
    Color(IglooColor),
    #[display("{_0}")]
    Date(IglooDate),
    #[display("{_0}")]
    Time(IglooTime),

    #[display("{_0:?}")]
    IntegerList(IglooIntegerList),
    #[display("{_0:?}")]
    RealList(IglooRealList),
    #[display("{_0:?}")]
    TextList(IglooTextList),
    #[display("{_0:?}")]
    BooleanList(IglooBooleanList),
    #[display("{_0:?}")]
    ColorList(IglooColorList),
    #[display("{_0:?}")]
    DateList(IglooDateList),
    #[display("{_0:?}")]
    TimeList(IglooTimeList),

    #[display("{_0}")]
    Enum(IglooEnumValue),
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
#[display("#{r:02x}{g:02x}{b:02x}")]
pub struct IglooColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
#[display("{year:04}-{month:02}-{day:02}")]
pub struct IglooDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
#[display("{hour:02}:{minute:02}:{second:02}")]
pub struct IglooTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl IglooValue {
    pub fn default(r#type: &IglooType) -> IglooValue {
        match r#type {
            IglooType::Integer => IglooValue::Integer(0),
            IglooType::Real => IglooValue::Real(0.0),
            IglooType::Text => IglooValue::Text(String::default()),
            IglooType::Boolean => IglooValue::Boolean(false),
            IglooType::Color => IglooValue::Color(IglooColor::default()),
            IglooType::Date => IglooValue::Date(IglooDate::default()),
            IglooType::Time => IglooValue::Time(IglooTime::default()),
            IglooType::IntegerList => IglooValue::IntegerList(Vec::new()),
            IglooType::RealList => IglooValue::RealList(Vec::new()),
            IglooType::TextList => IglooValue::TextList(Vec::new()),
            IglooType::BooleanList => IglooValue::BooleanList(Vec::new()),
            IglooType::ColorList => IglooValue::ColorList(Vec::new()),
            IglooType::DateList => IglooValue::DateList(Vec::new()),
            IglooType::TimeList => IglooValue::TimeList(Vec::new()),
            IglooType::Enum(t) => IglooValue::Enum(IglooEnumValue::default(t)),
        }
    }

    pub fn from_string(r#type: &IglooType, value: String) -> Option<IglooValue> {
        match r#type {
            IglooType::Integer => Some(IglooValue::Integer(value.parse().ok()?)),
            IglooType::Real => Some(IglooValue::Real(value.parse().ok()?)),
            IglooType::Text => Some(IglooValue::Text(value)),
            IglooType::Boolean => Some(IglooValue::Boolean(value.parse().ok()?)),
            IglooType::Color => Some(IglooValue::Color(value.try_into().ok()?)),
            IglooType::Date => Some(IglooValue::Date(value.try_into().ok()?)),
            IglooType::Time => Some(IglooValue::Time(value.try_into().ok()?)),
            IglooType::IntegerList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<i64>> = items.iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::IntegerList(list?))
            }
            IglooType::RealList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<f64>> = items.iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::RealList(list?))
            }
            IglooType::TextList => {
                let items = parse_list(&value)?;
                Some(IglooValue::TextList(items))
            }
            IglooType::BooleanList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<bool>> = items.iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::BooleanList(list?))
            }
            IglooType::ColorList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<IglooColor>> =
                    items.into_iter().map(|s| s.try_into().ok()).collect();
                Some(IglooValue::ColorList(list?))
            }
            IglooType::DateList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<IglooDate>> =
                    items.into_iter().map(|s| s.try_into().ok()).collect();
                Some(IglooValue::DateList(list?))
            }
            IglooType::TimeList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<IglooTime>> =
                    items.into_iter().map(|s| s.try_into().ok()).collect();
                Some(IglooValue::TimeList(list?))
            }
            IglooType::Enum(t) => Some(IglooValue::Enum(IglooEnumValue::from_string(t, value)?)),
        }
    }

    pub fn r#type(&self) -> IglooType {
        match self {
            IglooValue::Integer(_) => IglooType::Integer,
            IglooValue::Real(_) => IglooType::Real,
            IglooValue::Text(_) => IglooType::Text,
            IglooValue::Boolean(_) => IglooType::Boolean,
            IglooValue::Color(_) => IglooType::Color,
            IglooValue::Date(_) => IglooType::Date,
            IglooValue::Time(_) => IglooType::Time,
            IglooValue::IntegerList(_) => IglooType::IntegerList,
            IglooValue::RealList(_) => IglooType::RealList,
            IglooValue::TextList(_) => IglooType::TextList,
            IglooValue::BooleanList(_) => IglooType::BooleanList,
            IglooValue::ColorList(_) => IglooType::ColorList,
            IglooValue::DateList(_) => IglooType::DateList,
            IglooValue::TimeList(_) => IglooType::TimeList,
            IglooValue::Enum(t) => IglooType::Enum(t.get_type()),
        }
    }
}

/// "[1, 2, 3]"
pub fn parse_list(s: &str) -> Option<Vec<String>> {
    let s = s.trim();

    if !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }

    let inner = &s[1..s.len() - 1].trim();

    if inner.is_empty() {
        return Some(Vec::new());
    }

    let items: Vec<String> = inner
        .split(',')
        .map(|item| item.trim().to_string())
        .collect();

    Some(items)
}

impl IglooType {
    pub fn color(&self) -> &'static str {
        use IglooType::*;
        match self {
            Integer => "#3498db",
            Real => "#27ae60",
            Text => "#9b59b6",
            Boolean => "#e74c3c",
            Color => "#f39c12",
            Date => "#e67e22",
            Time => "#1abc9c",
            IntegerList => "#5dade2",
            RealList => "#58d68d",
            TextList => "#bb8fce",
            BooleanList => "#ec7063",
            ColorList => "#f8c471",
            DateList => "#f0b27a",
            TimeList => "#76d7c4",
            Enum(_) => "#95a5a6",
        }
    }

    pub fn can_cast(self, to: Self) -> bool {
        use IglooType::*;
        match (self, to) {
            (Integer, Real) => true,
            (Real, Integer) => true,

            (_, Text) => true,

            (Integer, Boolean) => true,
            (Boolean, Integer) => true,

            (IntegerList, RealList) => true,
            (RealList, IntegerList) => true,

            (IntegerList, TextList) => true,
            (RealList, TextList) => true,
            (BooleanList, TextList) => true,
            (ColorList, TextList) => true,
            (DateList, TextList) => true,
            (TimeList, TextList) => true,

            _ => false,
        }
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        if !self.can_cast(to) {
            return None;
        }
        Some(format!("Cast {self} to {to}"))
    }
}

impl TryFrom<String> for IglooColor {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.trim_start_matches('#');
        if s.len() != 6 {
            return Err(());
        }
        Ok(Self {
            r: u8::from_str_radix(&s[0..2], 16).map_err(|_| ())?,
            g: u8::from_str_radix(&s[2..4], 16).map_err(|_| ())?,
            b: u8::from_str_radix(&s[4..6], 16).map_err(|_| ())?,
        })
    }
}

impl TryFrom<String> for IglooDate {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // "YYYY-MM-DD"
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 3 {
            return Err(());
        }
        Ok(Self {
            year: parts[0].parse().map_err(|_| ())?,
            month: parts[1].parse().map_err(|_| ())?,
            day: parts[2].parse().map_err(|_| ())?,
        })
    }
}

impl TryFrom<String> for IglooTime {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // "HH:MM:SS"
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 3 {
            return Err(());
        }
        Ok(Self {
            hour: parts[0].parse().map_err(|_| ())?,
            minute: parts[1].parse().map_err(|_| ())?,
            second: parts[2].parse().map_err(|_| ())?,
        })
    }
}
