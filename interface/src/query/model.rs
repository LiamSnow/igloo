use crate::{
    Component, ComponentType, IglooType, IglooValue,
    id::{DeviceID, FloeID, GroupID},
    query::{DeviceSnapshot, EntitySnapshot, FloeSnapshot, GroupSnapshot},
    types::{agg::AggregationOp, compare::ComparisonOp},
};
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

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
    /// for queries (put, set, increment, watch entity/device)
    /// that don't return anything
    Ok,
    Devices(Vec<DeviceSnapshot>),
    Entities(Vec<EntitySnapshot>),
    Groups(Vec<GroupSnapshot>),
    Floes(Vec<FloeSnapshot>),
    Components(Vec<ComponentResult>),
    Aggregate(Option<IglooValue>),
    Count(usize),
    Ids(Vec<DeviceID>),
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum QueryResultType {
    /// for queries (put, set, increment, watch entity/device)
    /// that don't return anything
    Ok,
    Devices,
    Entities,
    Groups,
    Floes,
    Components(IglooType),
    Aggregate(IglooType),
    Count,
    Ids,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct ComponentResult {
    pub device: DeviceID,
    pub entity: String,
    pub value: IglooValue,
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

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
pub enum QueryAction {
    /// snapshot all floes
    #[display("snapshot Floes")]
    SnapshotFloes,
    /// snapshot all groups
    #[display("snapshot Groups")]
    SnapshotGroups,

    #[display("get ids")]
    GetIds,
    #[display("snapshot Devices")]
    SnapshotDevices,
    #[display("watch Devices")]
    WatchDevices,

    #[display("snapshot Entities")]
    SnapshotEntities,
    #[display("watch Entities")]
    WatchEntities,

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
