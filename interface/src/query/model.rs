use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::{
    Component, ComponentType, IglooType, IglooValue,
    id::{DeviceID, FloeID, GroupID},
    query::{AggregationOp, DeviceSnapshot, EntitySnapshot, FloeSnapshot, GroupSnapshot},
};

#[derive(Debug, Clone, PartialEq, Default, BorshSerialize, BorshDeserialize)]
pub struct Query {
    pub device_filter: Option<DeviceFilter>,
    pub entity_filter: Option<EntityFilter>,
    pub component_filter: Option<ComponentFilter>,
    pub limit: Option<usize>,
    pub tag: u32,
    pub action: QueryAction,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct QueryResult {
    pub value: QueryResultValue,
    pub tag: u32,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum QueryResultValue {
    None,
    Devices(Vec<DeviceSnapshot>),
    Device(DeviceSnapshot),
    Entities(Vec<EntitySnapshot>),
    Entity(EntitySnapshot),
    Groups(Vec<GroupSnapshot>),
    Floes(Vec<FloeSnapshot>),
    Components(Vec<ComponentResult>),
    Component(ComponentResult),
    Aggregate(IglooValue),
    Count(usize),
    Id(DeviceID),
    Ids(Vec<DeviceID>),
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct ComponentResult {
    pub device: DeviceID,
    pub entity: String,
    pub value: IglooValue,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum QueryResultType {
    None,
    Devices,
    Device,
    Entities,
    Entity,
    Groups,
    Floes,
    Components(IglooType),
    Component(IglooType),
    Aggregate(IglooType),
    Count,
    Id,
    Ids,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct WatchUpdate {
    pub value: WatchUpdateValue,
    pub tag: u32,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum WatchUpdateValue {
    Device(DeviceID),
    Entity(DeviceID, String),
    Component(ComponentResult),
    Aggregate(IglooValue),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum DeviceFilter {
    #[display("id {_0}")]
    Id(DeviceID),
    #[display("id in [{_0:?}]")]
    Ids(Vec<DeviceID>),
    #[display("named {_0}")]
    NameEquals(String),
    /// glob
    #[display("where name matches \"{_0}\"")]
    NameMatches(String),
    #[display("updated within {_0}s")]
    UpdatedWithinSeconds(u64),

    // TODO we could move these into group_filters and floe_filters
    // Allowing for snapshotting that way, but maybe thats overkill
    #[display("in group {_0}")]
    Group(GroupID),
    #[display("owned by {_0}")]
    Owner(FloeID),

    #[display("where entity count {_0} {_1}")]
    EntityCount(ComparisonOp, usize),
    #[display("with entity ({_0})")]
    HasEntity(EntityFilter),
    #[display("where all entities ({_0})")]
    AllEntities(EntityFilter),

    #[display("({_0} and {_1})")]
    And(Box<DeviceFilter>, Box<DeviceFilter>),
    #[display("({_0} or {_1})")]
    Or(Box<DeviceFilter>, Box<DeviceFilter>),
    #[display("not ({_0})")]
    Not(Box<DeviceFilter>),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum EntityFilter {
    #[display("named {_0}")]
    NameEquals(String),
    /// glob
    #[display("where name matches \"{_0}\"")]
    NameMatches(String),
    #[display("updated within {_0}s")]
    UpdatedWithinSeconds(u64),

    #[display("where component count {_0} {_1}")]
    ComponentCount(ComparisonOp, usize),
    #[display("with component ({_0})")]
    HasComponent(ComponentFilter),
    #[display("where all components ({_0})")]
    AllComponents(ComponentFilter),

    #[display("({_0} and {_1})")]
    And(Box<EntityFilter>, Box<EntityFilter>),
    #[display("({_0} or {_1})")]
    Or(Box<EntityFilter>, Box<EntityFilter>),
    #[display("not ({_0})")]
    Not(Box<EntityFilter>),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum ComponentFilter {
    #[display("{_0:?}")]
    Type(ComponentType),
    #[display("updated within {_0}s")]
    UpdatedWithinSeconds(u64),
    #[display("value {_0} {}", _1.inner_string().unwrap_or(format!("{_1:?}")))]
    Condition(ComparisonOp, Component),
    #[display("length {_1} {_2}")]
    ListLength(ComponentType, ComparisonOp, usize),

    #[display("({_0} and {_1})")]
    And(Box<ComponentFilter>, Box<ComponentFilter>),
    #[display("({_0} or {_1})")]
    Or(Box<ComponentFilter>, Box<ComponentFilter>),
    #[display("not ({_0})")]
    Not(Box<ComponentFilter>),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum ComparisonOp {
    #[display("==")]
    Eq,
    #[display("!=")]
    Neq,
    #[display(">")]
    Gt,
    #[display(">=")]
    Gte,
    #[display("<")]
    Lt,
    #[display("<=")]
    Lte,
    #[display("contains")]
    Contains,
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
pub enum QueryAction {
    /// snapshot all floes
    #[display("snapshot floes")]
    SnapshotFloes,
    /// snapshot all groups
    #[display("snapshot groups")]
    SnapshotGroups,

    #[display("get id")]
    GetId,
    #[display("snapshot device")]
    SnapshotDevice,
    #[display("watch entity")]
    WatchDevice,

    #[display("snapshot entity")]
    SnapshotEntity,
    #[display("watch entity")]
    WatchEntity,

    #[display("get")]
    #[default]
    Get,
    #[display("get {_0}")]
    GetAggregate(AggregationOp),
    #[display("set {_0:?}")]
    Set(Component),
    #[display("put {_0:?}")]
    Put(Component),
    #[display("increment {_0:?}")]
    Increment(Component),
    #[display("watch")]
    Watch,
    #[display("watch {_0}")]
    WatchAggregate(AggregationOp),

    #[display("count")]
    Count,

    /// used in dashboard bindings
    /// inherits action from custom
    /// element definition
    #[display("inherit")]
    Inherit,
}
