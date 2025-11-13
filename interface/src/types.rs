use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::{
    IGLOO_ENUMS, IglooEnumType, IglooEnumValue,
    compound::{IglooColor, IglooDate, IglooTime},
    id::{DeviceID, FloeID, GroupID},
    query::{DeviceSnapshot, EntitySnapshot, FloeSnapshot, GroupSnapshot},
};

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

pub type FloeIDList = Vec<FloeID>;
pub type DeviceIDList = Vec<DeviceID>;
pub type GroupIDList = Vec<GroupID>;
pub type FloeSnapshotList = Vec<FloeSnapshot>;
pub type DeviceSnapshotList = Vec<DeviceSnapshot>;
pub type GroupSnapshotList = Vec<GroupSnapshot>;
pub type EntitySnapshotList = Vec<EntitySnapshot>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
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
    Time, // TODO add duration?

    #[display("FloeID")]
    FloeID,
    #[display("DeviceID")]
    DeviceID,
    #[display("GroupID")]
    GroupID,
    #[display("FloeSnapshot")]
    FloeSnapshot,
    #[display("DeviceSnapshot")]
    DeviceSnapshot,
    #[display("GroupSnapshot")]
    GroupSnapshot,
    #[display("EntitySnapshot")]
    EntitySnapshot,

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

    #[display("FloeIDList")]
    FloeIDList,
    #[display("DeviceIDList")]
    DeviceIDList,
    #[display("GroupIDList")]
    GroupIDList,
    #[display("FloeSnapshotList")]
    FloeSnapshotList,
    #[display("DeviceSnapshotList")]
    DeviceSnapshotList,
    #[display("GroupSnapshotList")]
    GroupSnapshotList,
    #[display("EntitySnapshotList")]
    EntitySnapshotList,

    #[display("Enum")]
    Enum(IglooEnumType),
}

pub static IGLOO_TYPES: [IglooType; 28] = [
    IglooType::Integer,
    IglooType::Real,
    IglooType::Text,
    IglooType::Boolean,
    IglooType::Color,
    IglooType::Date,
    IglooType::Time,
    IglooType::FloeID,
    IglooType::DeviceID,
    IglooType::GroupID,
    IglooType::FloeSnapshot,
    IglooType::DeviceSnapshot,
    IglooType::GroupSnapshot,
    IglooType::EntitySnapshot,
    IglooType::IntegerList,
    IglooType::RealList,
    IglooType::TextList,
    IglooType::BooleanList,
    IglooType::ColorList,
    IglooType::DateList,
    IglooType::TimeList,
    IglooType::FloeIDList,
    IglooType::DeviceIDList,
    IglooType::GroupIDList,
    IglooType::FloeSnapshotList,
    IglooType::DeviceSnapshotList,
    IglooType::GroupSnapshotList,
    IglooType::EntitySnapshotList,
];

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
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

    #[display("{_0}")]
    FloeID(FloeID),
    #[display("{_0}")]
    DeviceID(DeviceID),
    #[display("{_0}")]
    GroupID(GroupID),
    #[display("{_0}")]
    FloeSnapshot(FloeSnapshot),
    #[display("{_0}")]
    DeviceSnapshot(DeviceSnapshot),
    #[display("{_0}")]
    GroupSnapshot(GroupSnapshot),
    #[display("{_0}")]
    EntitySnapshot(EntitySnapshot),

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

    #[display("{_0:?}")]
    FloeIDList(FloeIDList),
    #[display("{_0:?}")]
    DeviceIDList(DeviceIDList),
    #[display("{_0:?}")]
    GroupIDList(GroupIDList),
    #[display("{_0:?}")]
    FloeSnapshotList(FloeSnapshotList),
    #[display("{_0:?}")]
    DeviceSnapshotList(DeviceSnapshotList),
    #[display("{_0:?}")]
    GroupSnapshotList(GroupSnapshotList),
    #[display("{_0:?}")]
    EntitySnapshotList(EntitySnapshotList),

    #[display("{_0}")]
    Enum(IglooEnumValue),
}

impl IglooValue {
    pub fn default(r#type: &IglooType) -> IglooValue {
        match r#type {
            IglooType::Integer => IglooValue::Integer(0),
            IglooType::Real => IglooValue::Real(0.0),
            IglooType::Text => IglooValue::Text(String::with_capacity(20)),
            IglooType::Boolean => IglooValue::Boolean(false),
            IglooType::Color => IglooValue::Color(IglooColor::default()),
            IglooType::Date => IglooValue::Date(IglooDate::default()),
            IglooType::Time => IglooValue::Time(IglooTime::default()),

            //
            IglooType::FloeID => IglooValue::FloeID(FloeID::default()),
            IglooType::DeviceID => IglooValue::DeviceID(DeviceID::default()),
            IglooType::GroupID => IglooValue::GroupID(GroupID::default()),
            IglooType::FloeSnapshot => IglooValue::FloeSnapshot(FloeSnapshot::default()),
            IglooType::DeviceSnapshot => IglooValue::DeviceSnapshot(DeviceSnapshot::default()),
            IglooType::GroupSnapshot => IglooValue::GroupSnapshot(GroupSnapshot::default()),
            IglooType::EntitySnapshot => IglooValue::EntitySnapshot(EntitySnapshot::default()),

            //
            IglooType::IntegerList => IglooValue::IntegerList(Vec::with_capacity(10)),
            IglooType::RealList => IglooValue::RealList(Vec::with_capacity(10)),
            IglooType::TextList => IglooValue::TextList(Vec::with_capacity(10)),
            IglooType::BooleanList => IglooValue::BooleanList(Vec::with_capacity(10)),
            IglooType::ColorList => IglooValue::ColorList(Vec::with_capacity(10)),
            IglooType::DateList => IglooValue::DateList(Vec::with_capacity(10)),
            IglooType::TimeList => IglooValue::TimeList(Vec::with_capacity(10)),

            //
            IglooType::FloeIDList => IglooValue::FloeIDList(Vec::with_capacity(10)),
            IglooType::DeviceIDList => IglooValue::DeviceIDList(Vec::with_capacity(10)),
            IglooType::GroupIDList => IglooValue::GroupIDList(Vec::with_capacity(10)),
            IglooType::FloeSnapshotList => IglooValue::FloeSnapshotList(Vec::with_capacity(10)),
            IglooType::DeviceSnapshotList => IglooValue::DeviceSnapshotList(Vec::with_capacity(10)),
            IglooType::GroupSnapshotList => IglooValue::GroupSnapshotList(Vec::with_capacity(10)),
            IglooType::EntitySnapshotList => IglooValue::EntitySnapshotList(Vec::with_capacity(10)),

            //
            IglooType::Enum(t) => IglooValue::Enum(IglooEnumValue::default(t)),
        }
    }

    // TODO return Result instead of option
    pub fn from_string(r#type: &IglooType, value: String) -> Option<IglooValue> {
        match r#type {
            IglooType::Integer => Some(IglooValue::Integer(value.parse().ok()?)),
            IglooType::Real => Some(IglooValue::Real(value.parse().ok()?)),
            IglooType::Text => Some(IglooValue::Text(value)),
            IglooType::Boolean => Some(IglooValue::Boolean(value.parse().ok()?)),
            IglooType::Color => Some(IglooValue::Color(value.parse().ok()?)),
            IglooType::Date => Some(IglooValue::Date(value.parse().ok()?)),
            IglooType::Time => Some(IglooValue::Time(value.parse().ok()?)),

            //
            IglooType::FloeID => Some(IglooValue::FloeID(FloeID(value))),
            IglooType::DeviceID => Some(IglooValue::DeviceID(value.parse().ok()?)),
            IglooType::GroupID => Some(IglooValue::GroupID(value.parse().ok()?)),
            IglooType::FloeSnapshot => None,
            IglooType::DeviceSnapshot => None,
            IglooType::GroupSnapshot => None,
            IglooType::EntitySnapshot => None,

            //
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
                    items.into_iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::ColorList(list?))
            }
            IglooType::DateList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<IglooDate>> =
                    items.into_iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::DateList(list?))
            }
            IglooType::TimeList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<IglooTime>> =
                    items.into_iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::TimeList(list?))
            }

            //
            IglooType::FloeIDList => {
                let items = parse_list(&value)?;
                Some(IglooValue::FloeIDList(
                    items.into_iter().map(FloeID).collect(),
                ))
            }
            IglooType::DeviceIDList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<DeviceID>> = items.iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::DeviceIDList(list?))
            }
            IglooType::GroupIDList => {
                let items = parse_list(&value)?;
                let list: Option<Vec<GroupID>> = items.iter().map(|s| s.parse().ok()).collect();
                Some(IglooValue::GroupIDList(list?))
            }
            IglooType::FloeSnapshotList => None,
            IglooType::DeviceSnapshotList => None,
            IglooType::GroupSnapshotList => None,
            IglooType::EntitySnapshotList => None,

            //
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

            //
            IglooValue::FloeID(_) => IglooType::FloeID,
            IglooValue::DeviceID(_) => IglooType::DeviceID,
            IglooValue::GroupID(_) => IglooType::GroupID,
            IglooValue::FloeSnapshot(_) => IglooType::FloeSnapshot,
            IglooValue::DeviceSnapshot(_) => IglooType::DeviceSnapshot,
            IglooValue::GroupSnapshot(_) => IglooType::GroupSnapshot,
            IglooValue::EntitySnapshot(_) => IglooType::EntitySnapshot,

            //
            IglooValue::IntegerList(_) => IglooType::IntegerList,
            IglooValue::RealList(_) => IglooType::RealList,
            IglooValue::TextList(_) => IglooType::TextList,
            IglooValue::BooleanList(_) => IglooType::BooleanList,
            IglooValue::ColorList(_) => IglooType::ColorList,
            IglooValue::DateList(_) => IglooType::DateList,
            IglooValue::TimeList(_) => IglooType::TimeList,

            //
            IglooValue::FloeIDList(_) => IglooType::FloeIDList,
            IglooValue::DeviceIDList(_) => IglooType::DeviceIDList,
            IglooValue::GroupIDList(_) => IglooType::GroupIDList,
            IglooValue::FloeSnapshotList(_) => IglooType::FloeSnapshotList,
            IglooValue::DeviceSnapshotList(_) => IglooType::DeviceSnapshotList,
            IglooValue::GroupSnapshotList(_) => IglooType::GroupSnapshotList,
            IglooValue::EntitySnapshotList(_) => IglooType::EntitySnapshotList,

            //
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

            //
            FloeID => "#e91e63",
            DeviceID => "#16a085",
            GroupID => "#f1c40f",
            FloeSnapshot => "#8e44ad",
            DeviceSnapshot => "#2c3e50",
            GroupSnapshot => "#d35400",
            EntitySnapshot => "#7f8c8d",

            //
            IntegerList => "#5dade2",
            RealList => "#58d68d",
            TextList => "#bb8fce",
            BooleanList => "#ec7063",
            ColorList => "#f8c471",
            DateList => "#f0b27a",
            TimeList => "#76d7c4",

            //
            FloeIDList => "#f48fb1",
            DeviceIDList => "#7dcea0",
            GroupIDList => "#f9e79f",
            FloeSnapshotList => "#af7ac5",
            DeviceSnapshotList => "#5d6d7e",
            GroupSnapshotList => "#e59866",
            EntitySnapshotList => "#bdc3c7",

            Enum(_) => "#95a5a6",
        }
    }

    pub fn can_cast(self, to: Self) -> bool {
        use IglooType::*;
        matches!(
            (self, to),
            (Integer, Real)
                | (Real, Integer)
                | (_, Text)
                | (Integer, Boolean)
                | (Boolean, Integer)
                | (IntegerList, RealList)
                | (RealList, IntegerList)
                | (IntegerList, TextList)
                | (RealList, TextList)
                | (BooleanList, TextList)
                | (ColorList, TextList)
                | (DateList, TextList)
                | (TimeList, TextList)
        )
    }

    pub fn cast_name(self, to: Self) -> Option<String> {
        if !self.can_cast(to) {
            return None;
        }
        Some(format!("Cast {self} to {to}"))
    }

    pub fn all() -> Vec<Self> {
        let mut res = IGLOO_TYPES.to_vec();
        for e in IGLOO_ENUMS {
            res.push(Self::Enum(e));
        }
        res
    }
}
