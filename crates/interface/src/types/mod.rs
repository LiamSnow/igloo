use crate::{
    generated::id::{DeviceId, ExtensionId, GroupId},
    query::{DeviceSnapshot, EntitySnapshot, ExtensionSnapshot, GroupSnapshot},
};

pub mod agg;
pub mod cast;
pub mod compare;
pub mod compound;
pub use compound::*;
use serde::{Deserialize, Serialize};
pub mod math;

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

pub type ExtensionIdList = Vec<ExtensionId>;
pub type DeviceIdList = Vec<DeviceId>;
pub type GroupIdList = Vec<GroupId>;
pub type ExtensionSnapshotList = Vec<ExtensionSnapshot>;
pub type DeviceSnapshotList = Vec<DeviceSnapshot>;
pub type GroupSnapshotList = Vec<GroupSnapshot>;
pub type EntitySnapshotList = Vec<EntitySnapshot>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "subtype")]
pub enum IglooType {
    Integer,
    Real,
    Text,
    Boolean,
    Color,
    Date,
    Time, // TODO add duration?

    ExtensionId,
    DeviceId,
    GroupId,
    ExtensionSnapshot,
    DeviceSnapshot,
    GroupSnapshot,
    EntitySnapshot,

    IntegerList,
    RealList,
    TextList,
    BooleanList,
    ColorList,
    DateList,
    TimeList,

    ExtensionIdList,
    DeviceIdList,
    GroupIdList,
    ExtensionSnapshotList,
    DeviceSnapshotList,
    GroupSnapshotList,
    EntitySnapshotList,

    Enum(String),
}

pub static IGLOO_TYPES: [IglooType; 28] = [
    IglooType::Integer,
    IglooType::Real,
    IglooType::Text,
    IglooType::Boolean,
    IglooType::Color,
    IglooType::Date,
    IglooType::Time,
    IglooType::ExtensionId,
    IglooType::DeviceId,
    IglooType::GroupId,
    IglooType::ExtensionSnapshot,
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
    IglooType::ExtensionIdList,
    IglooType::DeviceIdList,
    IglooType::GroupIdList,
    IglooType::ExtensionSnapshotList,
    IglooType::DeviceSnapshotList,
    IglooType::GroupSnapshotList,
    IglooType::EntitySnapshotList,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IglooValue {
    Integer(IglooInteger),
    Real(IglooReal),
    Text(IglooText),
    Boolean(IglooBoolean),
    Color(IglooColor),
    Date(IglooDate),
    Time(IglooTime),

    ExtensionId(ExtensionId),
    DeviceId(DeviceId),
    GroupId(GroupId),
    ExtensionSnapshot(ExtensionSnapshot),
    DeviceSnapshot(DeviceSnapshot),
    GroupSnapshot(GroupSnapshot),
    EntitySnapshot(EntitySnapshot),

    IntegerList(IglooIntegerList),
    RealList(IglooRealList),
    TextList(IglooTextList),
    BooleanList(IglooBooleanList),
    ColorList(IglooColorList),
    DateList(IglooDateList),
    TimeList(IglooTimeList),

    ExtensionIdList(ExtensionIdList),
    DeviceIdList(DeviceIdList),
    GroupIdList(GroupIdList),
    ExtensionSnapshotList(ExtensionSnapshotList),
    DeviceSnapshotList(DeviceSnapshotList),
    GroupSnapshotList(GroupSnapshotList),
    EntitySnapshotList(EntitySnapshotList),

    Enum(String),
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
            IglooType::ExtensionId => IglooValue::ExtensionId(ExtensionId::default()),
            IglooType::DeviceId => IglooValue::DeviceId(DeviceId::default()),
            IglooType::GroupId => IglooValue::GroupId(GroupId::default()),
            IglooType::ExtensionSnapshot => {
                IglooValue::ExtensionSnapshot(ExtensionSnapshot::default())
            }
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
            IglooType::ExtensionIdList => IglooValue::ExtensionIdList(Vec::with_capacity(10)),
            IglooType::DeviceIdList => IglooValue::DeviceIdList(Vec::with_capacity(10)),
            IglooType::GroupIdList => IglooValue::GroupIdList(Vec::with_capacity(10)),
            IglooType::ExtensionSnapshotList => {
                IglooValue::ExtensionSnapshotList(Vec::with_capacity(10))
            }
            IglooType::DeviceSnapshotList => IglooValue::DeviceSnapshotList(Vec::with_capacity(10)),
            IglooType::GroupSnapshotList => IglooValue::GroupSnapshotList(Vec::with_capacity(10)),
            IglooType::EntitySnapshotList => IglooValue::EntitySnapshotList(Vec::with_capacity(10)),

            //
            IglooType::Enum(t) => IglooValue::Enum(IglooEnumValue::default(t)),
        }
    }

    // TODO return Result instead of option
    // pub fn from_string(r#type: &IglooType, value: String) -> Option<IglooValue> {
    //     match r#type {
    //         IglooType::Integer => Some(IglooValue::Integer(value.parse().ok()?)),
    //         IglooType::Real => Some(IglooValue::Real(value.parse().ok()?)),
    //         IglooType::Text => Some(IglooValue::Text(value)),
    //         IglooType::Boolean => Some(IglooValue::Boolean(value.parse().ok()?)),
    //         IglooType::Color => Some(IglooValue::Color(value.parse().ok()?)),
    //         IglooType::Date => Some(IglooValue::Date(value.parse().ok()?)),
    //         IglooType::Time => Some(IglooValue::Time(value.parse().ok()?)),

    //         //
    //         IglooType::ExtensionId => Some(IglooValue::ExtensionId(ExtensionId(value))),
    //         IglooType::DeviceId => Some(IglooValue::DeviceId(DeviceId::new(value.parse().ok()?))),
    //         IglooType::GroupId => Some(IglooValue::GroupId(GroupId::ewvalue.parse().ok()?)),
    //         IglooType::ExtensionSnapshot => None,
    //         IglooType::DeviceSnapshot => None,
    //         IglooType::GroupSnapshot => None,
    //         IglooType::EntitySnapshot => None,

    //         //
    //         IglooType::IntegerList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<i64>> = items.iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::IntegerList(list?))
    //         }
    //         IglooType::RealList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<f64>> = items.iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::RealList(list?))
    //         }
    //         IglooType::TextList => {
    //             let items = parse_list(&value)?;
    //             Some(IglooValue::TextList(items))
    //         }
    //         IglooType::BooleanList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<bool>> = items.iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::BooleanList(list?))
    //         }
    //         IglooType::ColorList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<IglooColor>> =
    //                 items.into_iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::ColorList(list?))
    //         }
    //         IglooType::DateList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<IglooDate>> =
    //                 items.into_iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::DateList(list?))
    //         }
    //         IglooType::TimeList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<IglooTime>> =
    //                 items.into_iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::TimeList(list?))
    //         }

    //         //
    //         IglooType::ExtensionIdList => {
    //             let items = parse_list(&value)?;
    //             Some(IglooValue::ExtensionIdList(
    //                 items.into_iter().map(ExtensionId).collect(),
    //             ))
    //         }
    //         IglooType::DeviceIdList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<DeviceId>> = items.iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::DeviceIdList(list?))
    //         }
    //         IglooType::GroupIdList => {
    //             let items = parse_list(&value)?;
    //             let list: Option<Vec<GroupId>> = items.iter().map(|s| s.parse().ok()).collect();
    //             Some(IglooValue::GroupIdList(list?))
    //         }
    //         IglooType::ExtensionSnapshotList => None,
    //         IglooType::DeviceSnapshotList => None,
    //         IglooType::GroupSnapshotList => None,
    //         IglooType::EntitySnapshotList => None,

    //         //
    //         IglooType::Enum(t) => Some(IglooValue::Enum(IglooEnumValue::from_string(t, value)?)),
    //     }
    // }

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
            IglooValue::ExtensionId(_) => IglooType::ExtensionId,
            IglooValue::DeviceId(_) => IglooType::DeviceId,
            IglooValue::GroupId(_) => IglooType::GroupId,
            IglooValue::ExtensionSnapshot(_) => IglooType::ExtensionSnapshot,
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
            IglooValue::ExtensionIdList(_) => IglooType::ExtensionIdList,
            IglooValue::DeviceIdList(_) => IglooType::DeviceIdList,
            IglooValue::GroupIdList(_) => IglooType::GroupIdList,
            IglooValue::ExtensionSnapshotList(_) => IglooType::ExtensionSnapshotList,
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
            ExtensionId => "#e91e63",
            DeviceId => "#16a085",
            GroupId => "#f1c40f",
            ExtensionSnapshot => "#8e44ad",
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
            ExtensionIdList => "#f48fb1",
            DeviceIdList => "#7dcea0",
            GroupIdList => "#f9e79f",
            ExtensionSnapshotList => "#af7ac5",
            DeviceSnapshotList => "#5d6d7e",
            GroupSnapshotList => "#e59866",
            EntitySnapshotList => "#bdc3c7",

            Enum(_) => "#95a5a6",
        }
    }

    // pub fn all() -> Vec<Self> {
    //     let mut res = IGLOO_TYPES.to_vec();
    //     for e in IGLOO_ENUMS {
    //         res.push(Self::Enum(e));
    //     }
    //     res
    // }
}
