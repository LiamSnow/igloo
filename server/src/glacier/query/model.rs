use igloo_interface::{Component, ComponentType};
use tokio::sync::oneshot;

use crate::glacier::tree::{DeviceID, FloeID, FloeRef, ZoneID};

/// Request a Query from the Query Engine
#[derive(Debug)]
pub struct Query {
    pub filter: Option<QueryFilter>,
    pub target: QueryTarget,
    pub kind: QueryKind,
}

#[derive(Debug)]
pub enum QueryKind {
    Set(Vec<Component>),
    GetOne(oneshot::Sender<Option<Component>>, ComponentType),
    GetAll(oneshot::Sender<Vec<Component>>, ComponentType),
    GetAvg(oneshot::Sender<Option<Component>>, ComponentType),
    // WatchOne(mpsc::Sender<()>, ComponentType),
    // WatchAll(mpsc::Sender<()>, ComponentType),
    // WatchAvg(mpsc::Sender<()>, ComponentType),
    Snapshot(oneshot::Sender<Snapshot>),
}

pub enum WatchTarget {
    All,
    Zone(ZoneID),
    Device(DeviceID),
    Entity(DeviceID, usize),
}

#[derive(Debug, Clone)]
pub enum QueryTarget {
    All,
    Zone(ZoneID),
    Device(DeviceID),
    /// Device ID, Entity Name
    Entity(DeviceID, String),
}

#[derive(Debug, Clone)]
pub enum QueryFilter {
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

#[allow(dead_code)]
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
    pub zones: Vec<ZoneSnapshot>,
    pub devices: Vec<DeviceSnapshot>,
}

#[derive(Debug)]
pub struct ZoneSnapshot {
    pub id: ZoneID,
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
