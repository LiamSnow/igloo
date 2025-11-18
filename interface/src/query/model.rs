use crate::{
    Component, ComponentType, IglooType, IglooValue,
    id::{DeviceID, FloeID, GroupID},
    query::{DeviceSnapshot, EntitySnapshot, FloeSnapshot, GroupSnapshot, display::pluralize},
    types::{agg::AggregationOp, compare::ComparisonOp, math::MathOp},
};
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Default, BorshSerialize, BorshDeserialize)]
pub struct Query {
    pub action: QueryAction,
    pub target: QueryTarget,
    pub floe_filter: Option<FloeFilter>,
    pub group_filter: Option<GroupFilter>,
    pub device_filter: Option<DeviceFilter>,
    pub entity_filter: Option<EntityFilter>,
    pub limit: Option<usize>,
    pub tag: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Display, BorshSerialize, BorshDeserialize)]
pub enum QueryTarget {
    #[display("Floes")]
    Floes,
    #[display("Groups")]
    Groups,
    #[default]
    #[display("Devices")]
    Devices,
    #[display("Entities")]
    Entities,
    #[display("{}", pluralize(format!("{_0:?}")))]
    Components(ComponentType),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum FloeFilter {
    #[display("id {_0}")]
    Id(FloeID),
    #[display("id in [{_0:?}]")]
    Ids(HashSet<FloeID>),
    /// glob
    #[display("where id matches \"{_0}\"")]
    IdMatches(String),

    #[display("where device count {_0} {_1}")]
    DeviceCount(ComparisonOp, usize),
    #[display("with device ({_0})")]
    HasDevice(DeviceFilter),
    #[display("where all devices ({_0})")]
    AllDevices(DeviceFilter),

    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" and "))]
    All(Vec<FloeFilter>),
    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" or "))]
    Any(Vec<FloeFilter>),
    #[display("not ({_0})")]
    Not(Box<FloeFilter>),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum GroupFilter {
    #[display("with id {_0}")]
    Id(GroupID),
    #[display("with id in [{_0:?}]")]
    Ids(HashSet<GroupID>),
    #[display("named {_0}")]
    NameEquals(String),
    /// glob
    #[display("where name matches \"{_0}\"")]
    NameMatches(String),

    #[display("where device count {_0} {_1}")]
    DeviceCount(ComparisonOp, usize),
    #[display("with device ({_0})")]
    HasDevice(DeviceFilter),
    #[display("where all devices ({_0})")]
    AllDevices(DeviceFilter),

    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" and "))]
    All(Vec<GroupFilter>),
    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" or "))]
    Any(Vec<GroupFilter>),
    #[display("not ({_0})")]
    Not(Box<GroupFilter>),
}

#[derive(Debug, Clone, PartialEq, Display, BorshSerialize, BorshDeserialize)]
pub enum DeviceFilter {
    #[display("with id {_0}")]
    Id(DeviceID),
    #[display("with id in [{_0:?}]")]
    Ids(HashSet<DeviceID>),
    #[display("named {_0}")]
    NameEquals(String),
    /// glob
    #[display("where name matches \"{_0}\"")]
    NameMatches(String),
    #[display("updated within {_0}s")]
    UpdatedWithinSeconds(u64),

    #[display("where entity count {_0} {_1}")]
    EntityCount(ComparisonOp, usize),
    #[display("with entity ({_0})")]
    HasEntity(EntityFilter),
    #[display("where all entities ({_0})")]
    AllEntities(EntityFilter),

    #[display("with components in {_0:?}")]
    HasAll(Vec<ComponentType>),

    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" and "))]
    All(Vec<DeviceFilter>),
    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" or "))]
    Any(Vec<DeviceFilter>),
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
    #[display("{:?} {_0} {}", _1.get_type(), _1.inner_string().unwrap_or(format!("{_1:?}")))]
    Condition(ComparisonOp, Component),
    #[display("with {_0:?}")]
    Has(ComponentType),
    #[display("with all in {_0:?}")]
    HasAll(Vec<ComponentType>),
    #[display("with any in {_0:?}")]
    HasAny(Vec<ComponentType>),

    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" and "))]
    All(Vec<EntityFilter>),
    #[display("{}", _0.iter().map(|f| format!("({f})")).collect::<Vec<_>>().join(" or "))]
    Any(Vec<EntityFilter>),
    #[display("not ({_0})")]
    Not(Box<EntityFilter>),
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
pub enum QueryAction {
    /// Read the value of all matching QueryTargets
    #[display("get")]
    Get,
    /// Compute the aggregate (ex. mean) value of all matching components
    /// Must be QueryTarget::Components, Componenent must be aggregatable
    #[display("get {_0} of")]
    GetAggregate(AggregationOp),
    /// Get the IDs of all matching targets
    #[display("get ids of")]
    #[default]
    GetIds,

    /// Receive update on every change of QueryTarget
    #[display("watch")]
    Watch,
    /// Continously compute the aggregate (ex. mean) value of all matching components
    /// On every change to those components
    /// Must be QueryTarget::Components, Componenent must be aggregatable
    #[display("watch {_0}")]
    WatchAggregate(AggregationOp),

    /// Set the value of a component
    /// Must be QueryTarget::Components
    #[display("set {_0}")]
    Set(IglooValue),
    /// Put a component on an Entity or set its value
    /// Must be QueryTarget::Components
    #[display("put {_0}")]
    Put(IglooValue),
    /// Apply operation on all components
    /// Must be QueryTarget::Components
    #[display("{_0}")]
    Apply(MathOp),

    /// Count the number of results
    #[display("count")]
    Count,

    /// Used in dashboard bindings inherits action from custom element definition
    /// WARN: Cannot evaluate. Must merge first.
    #[display("inherit")]
    Inherit,
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

    FloeIds(Vec<FloeID>),
    GroupIds(Vec<GroupID>),
    DeviceIds(Vec<DeviceID>),
    EntityIds(Vec<(String, usize)>),
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

    FloeIds,
    GroupIds,
    DeviceIds,
    EntityIds,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct ComponentResult {
    pub device: DeviceID,
    pub entity: usize,
    pub value: IglooValue,
}
