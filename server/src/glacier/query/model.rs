use borsh::{BorshDeserialize, BorshSerialize};
use igloo_interface::{Component, ComponentType};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use tokio::sync::{mpsc, oneshot};

use crate::glacier::tree::{DeviceID, FloeID, FloeRef, GroupID};

/// Request a Query from the Query Engine
#[derive(Debug)]
pub struct Query {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub kind: QueryKind,
}

#[derive(Debug)]
pub enum QueryKind {
    Set(SmallVec<[Component; 2]>),

    GetOne(oneshot::Sender<Option<OneQueryResult>>, ComponentType),
    GetAll(oneshot::Sender<GetAllQueryResult>, ComponentType),
    GetAvg(oneshot::Sender<Option<Component>>, ComponentType),

    /// prefix/ID, recver, ct
    WatchAll(u32, mpsc::Sender<PrefixedOneQueryResult>, ComponentType),
    // WatchOne
    // WatchAvg(mpsc::Sender<()>, ComponentType),
    Snapshot(oneshot::Sender<Snapshot>),
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct SetQuery {
    pub filter: QueryFilter,
    pub target: QueryTarget,
    pub values: Vec<Component>,
}

// TODO FIXME move
impl SetQuery {
    pub fn to_query(self) -> Query {
        Query {
            filter: self.filter,
            target: self.target,
            kind: QueryKind::Set(self.values.into()),
        }
    }
}

pub type PrefixedOneQueryResult = (u32, DeviceID, usize, Component);
pub type OneQueryResult = (DeviceID, usize, Component);
pub type GetAllQueryResult = FxHashMap<DeviceID, FxHashMap<usize, Component>>;

#[derive(Debug, Clone)]
pub struct WatchQuery {
    pub prefix: u32,
    pub filter: QueryFilter,
    pub tx: mpsc::Sender<PrefixedOneQueryResult>,
    pub gid: Option<GroupID>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum QueryTarget {
    All,
    Group(GroupID),
    Device(DeviceID),
    /// Device ID, Entity Name
    Entity(DeviceID, String),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
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
