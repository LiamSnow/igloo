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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OneShotQuery {
    Extension(ExtensionQuery),
    Group(GroupQuery),
    Device(DeviceQuery),
    Entity(EntityQuery),
    Component(ComponentQuery),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtensionQuery {
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: IDFilter<ExtensionID>,
    pub action: ExtensionAction,
    #[cfg_attr(feature = "serde", serde(default))]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExtensionAction {
    GetID,
    Snapshot,

    IsAttached,

    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupQuery {
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: IDFilter<GroupID>,
    pub action: GroupAction,
    #[cfg_attr(feature = "serde", serde(default))]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GroupAction {
    GetID,
    Snapshot,

    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceQuery {
    #[cfg_attr(feature = "serde", serde(default))]
    pub filter: DeviceFilter,
    pub action: DeviceAction,
    #[cfg_attr(feature = "serde", serde(default))]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceAction {
    GetID,
    /// true=include entity snapshots
    Snapshot(bool),
    IsAttached,

    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityQuery {
    #[cfg_attr(feature = "serde", serde(default))]
    pub device_filter: DeviceFilter,
    #[cfg_attr(feature = "serde", serde(default))]
    pub entity_filter: EntityFilter,
    pub action: EntityAction,
    #[cfg_attr(feature = "serde", serde(default))]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntityAction {
    Snapshot,
    Count,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentQuery {
    #[cfg_attr(feature = "serde", serde(default))]
    pub device_filter: DeviceFilter,
    #[cfg_attr(feature = "serde", serde(default))]
    pub entity_filter: EntityFilter,
    pub action: ComponentAction,
    pub component: ComponentType,
    #[cfg_attr(feature = "serde", serde(default))]
    pub post_op: Option<AggregationOp>,
    /// includes (DeviceID, EntityName) for each response
    /// R::ComponentValueWithParents instead of R::ComponentValue
    #[cfg_attr(feature = "serde", serde(default))]
    pub include_parents: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComponentAction {
    GetValue,

    Set(IglooValue),
    Put(IglooValue),
    Apply(MathOp),

    Count,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IDFilter<T> {
    #[default]
    Any,
    Is(T),
    OneOf(Vec<T>),
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct DeviceFilter {
    pub id: IDFilter<DeviceID>,
    pub owner: IDFilter<ExtensionID>,
    pub group: DeviceGroupFilter,

    pub entity_count: Option<(ComparisonOp, usize)>,

    /// seconds
    pub last_update: Option<(ComparisonOp, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeviceGroupFilter {
    #[default]
    Any,
    In(GroupID),
    InAny(Vec<GroupID>),
    InAll(Vec<GroupID>),
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeFilter {
    With(ComponentType),
    Without(ComponentType),
    And(Vec<TypeFilter>),
    Or(Vec<TypeFilter>),
    Not(Box<TypeFilter>),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValueFilter {
    If(ComparisonOp, Component),
    And(Vec<ValueFilter>),
    Or(Vec<ValueFilter>),
    Not(Box<ValueFilter>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
