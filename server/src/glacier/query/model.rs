use igloo_interface::{Component, ComponentType};
use rustc_hash::FxHashSet;
use tokio::sync::oneshot;

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
    // WatchGet(mpsc::Sender<()>, ComponentType),
    // WatchAvg(mpsc::Sender<()>, ComponentType),
    Snapshot(oneshot::Sender<Snapshot>),
}

pub enum WatchTarget {
    All,
    /// Floe idx, Device idx
    Devices(FxHashSet<(u16, u16)>),
    /// Floe idx, Device idx, Entity idx
    Entity(u16, u16, u16),
}

#[derive(Debug, Clone)]
pub enum QueryTarget {
    All,
    /// Zone ID
    Zone(String),
    /// Floe ID, Device ID
    Device(String, String),
    /// Floe ID, Device ID, Entity Name
    Entity(String, String, String),
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
    pub id: String,
    pub idx: u16,
    pub name: String,
    pub disabled: bool,
    // TODO devices
}

#[derive(Debug)]
pub struct FloeSnapshot {
    pub id: String,
    pub idx: u16,
    pub max_supported_component: u16,
}

#[derive(Debug)]
pub struct DeviceSnapshot {
    pub id: String,
    pub idx: Option<u16>,
    pub name: String,
    pub floe_id: String,
    /// None if not registered right now
    pub floe_idx: Option<u16>,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Debug)]
pub struct EntitySnapshot {
    pub name: String,
    pub components: Vec<Component>,
}
