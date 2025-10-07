use igloo_interface::{Component, ComponentType};
use std::time::Instant;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::glacier::floe::Device;

/// Request a Query from the Query Engine
#[derive(Debug)]
pub struct Query {
    pub filter: QueryFilter,
    pub area: Area,
    pub kind: QueryKind,
    pub started_at: Instant,
}

#[derive(Debug, Clone)]
pub enum QueryKind {
    Set(Vec<Component>),
    // WatchGet(mpsc::Sender<()>, ComponentType),
    // WatchAvg(mpsc::Sender<()>, ComponentType),
    Snapshot(mpsc::Sender<SnapshotPart>),
}

/// Procesed [GlobalQueryRequest], dispatched to each Floe
#[derive(Debug)]
pub struct LocalQuery {
    pub filter: QueryFilter,
    pub area: LocalArea,
    pub kind: LocalQueryKind,
    pub started_at: Instant,
}

#[derive(Debug, Clone)]
pub enum LocalQueryKind {
    Set(Vec<Component>),
    // WatchGet(mpsc::Sender<()>, ComponentType),
    // WatchAvg(mpsc::Sender<()>, ComponentType),
    /// Device idx -> (id, name)
    Snapshot(Vec<(String, String)>, mpsc::Sender<SnapshotPart>),
}

pub struct SnapshotPart {
    pub floe_idx: usize,
    pub floe_name: String,
    /// device ID, name, data
    pub devices: Vec<(String, String, Device)>,
}

#[derive(Debug, Clone)]
pub enum Area {
    All,
    /// Zone ID
    Zone(Uuid),
    /// Global Device ID
    Device(String),
    /// Global Device ID, Entity Name
    Entity(String, String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum LocalArea {
    All,
    Device(u16),
    Entity(u16, u16),
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
