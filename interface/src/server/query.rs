use borsh::{BorshDeserialize, BorshSerialize};

use crate::{Component, ComponentType, DeviceID, FloeID, FloeRef, GroupID};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct SetQuery {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub values: Vec<Component>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum QueryTarget {
    All,
    Group(GroupID),
    Device(DeviceID),
    /// Device ID, Entity Name
    Entity(DeviceID, String),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum QueryFilter {
    /// no filter
    None,
    /// exclude entities that don't also have this component
    With(ComponentType),
    /// exclude entities that have this component
    Without(ComponentType),
    /// both queries must be true
    And(Box<(QueryFilter, QueryFilter)>),
    /// either query must be true
    Or(Box<(QueryFilter, QueryFilter)>),
    // Condition(ComponentType, Operator, Component),
    // for refering to parts of components, ex. color.r
    // NestedCondition(Vec<PathSegment>, Operator, Component),
    // TODO think it would be cool to have filters for entity names
    // IE `RGBCT_Bulb*`
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}

#[derive(Debug, Default)]
pub struct Snapshot {
    pub floes: Vec<FloeSnapshot>,
    pub groups: Vec<GroupSnapshot>,
    pub devices: Vec<DeviceSnapshot>,
}

#[derive(Debug)]
pub struct GroupSnapshot {
    pub id: GroupID,
    pub name: String,
    pub devices: Vec<DeviceID>,
}

#[derive(Debug)]
pub struct FloeSnapshot {
    pub id: FloeID,
    pub fref: FloeRef,
    pub max_supported_component: u16,
}

#[derive(Debug)]
pub struct DeviceSnapshot {
    pub id: DeviceID,
    pub name: String,
    pub owner: FloeID,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Debug)]
pub struct EntitySnapshot {
    pub name: String,
    pub components: Vec<Component>,
}
