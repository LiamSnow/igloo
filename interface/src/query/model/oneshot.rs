use crate::{
    Component, ComponentType, IglooType, IglooValue,
    id::{DeviceID, EntityID, ExtensionID, GroupID},
    query::{DeviceSnapshot, EntitySnapshot, ExtensionSnapshot, GroupSnapshot},
    types::{agg::AggregationOp, compare::ComparisonOp, math::MathOp},
};
use bincode::{Decode, Encode};

// TODO
//  - now that we use Bincode, we should add SmallVecs
//  - Pyo3 python package
//  - temporal component value queries (requires changes to device tree first)

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum OneShotQuery {
    Extension(ExtensionQuery),
    Group(GroupQuery),
    Device(DeviceQuery),
    Entity(EntityQuery),
    Component(ComponentQuery),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct ExtensionQuery {
    pub id: IDFilter<ExtensionID>,
    pub action: ExtensionAction,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ExtensionAction {
    GetID,
    Snapshot,

    IsAttached,

    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct GroupQuery {
    pub id: IDFilter<GroupID>,
    pub action: GroupAction,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum GroupAction {
    GetID,
    Snapshot,

    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct DeviceQuery {
    pub filter: DeviceFilter,
    pub action: DeviceAction,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum DeviceAction {
    GetID,
    /// true=include entity snapshots
    Snapshot(bool),
    IsAttached,

    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct EntityQuery {
    pub device_filter: DeviceFilter,
    pub entity_filter: EntityFilter,
    pub action: EntityAction,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum EntityAction {
    Snapshot,
    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct ComponentQuery {
    pub device_filter: DeviceFilter,
    pub entity_filter: EntityFilter,
    pub action: ComponentAction,
    pub component: ComponentType,
    pub post_op: Option<AggregationOp>,
    /// includes (DeviceID, EntityName) for each response
    /// R::ComponentValueWithParents instead of R::ComponentValue
    pub include_parents: bool,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ComponentAction {
    GetValue,

    Set(IglooValue),
    Put(IglooValue),
    Apply(MathOp),

    Count,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
pub enum IDFilter<T> {
    #[default]
    Any,
    Is(T),
    OneOf(Vec<T>),
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct DeviceFilter {
    pub id: IDFilter<DeviceID>,
    pub owner: IDFilter<ExtensionID>,
    pub group: DeviceGroupFilter,

    pub entity_count: Option<(ComparisonOp, usize)>,

    /// seconds
    pub last_update: Option<(ComparisonOp, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
pub enum DeviceGroupFilter {
    #[default]
    Any,
    In(GroupID),
    InAny(Vec<GroupID>),
    InAll(Vec<GroupID>),
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct EntityFilter {
    pub id: EntityIDFilter,

    /// Optimized by using device presense
    /// to reduce scanning a device's entities
    /// that dont have any of this component
    pub type_filter: Option<TypeFilter>,
    pub value_filter: Option<ValueFilter>,

    /// seconds
    pub last_update: Option<(ComparisonOp, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum TypeFilter {
    With(ComponentType),
    Without(ComponentType),
    And(Vec<TypeFilter>),
    Or(Vec<TypeFilter>),
    Not(Box<TypeFilter>),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ValueFilter {
    If(ComparisonOp, Component),
    And(Vec<ValueFilter>),
    Or(Vec<ValueFilter>),
    Not(Box<ValueFilter>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
pub enum EntityIDFilter {
    #[default]
    Any,
    Is(String),
    OneOf(Vec<String>),
    /// glob pattern
    Matches(String),
}

// -- Responses

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum QueryResult {
    /// For Put, Set, Apply
    Ok,

    ExtensionId(Vec<ExtensionID>),
    ExtensionSnapshot(Vec<ExtensionSnapshot>),
    ExtensionAttached(Vec<(ExtensionID, bool)>),

    GroupId(Vec<GroupID>),
    GroupSnapshot(Vec<GroupSnapshot>),

    DeviceId(Vec<DeviceID>),
    DeviceSnapshot(Vec<DeviceSnapshot>),
    DeviceAttached(Vec<(DeviceID, bool)>),

    EntitySnapshot(Vec<EntitySnapshot>),

    Aggregate(Option<IglooValue>),
    ComponentValue(Vec<IglooValue>),
    ComponentValueWithParents(Vec<(DeviceID, EntityID, IglooValue)>),

    Count(usize),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum QueryResultType {
    Ok,

    ExtensionID,
    ExtensionSnapshot,
    ExtensionAttached,

    GroupID,
    GroupSnapshot,

    DeviceID,
    DeviceSnapshot,
    DeviceAttached,

    EntitySnapshot,

    Aggregate(IglooType),
    ComponentValue(IglooType),
    ComponentValueWithParents(IglooType),

    Count,
}
