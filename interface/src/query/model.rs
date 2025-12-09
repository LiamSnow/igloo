use crate::{
    Component, ComponentType, IglooType, IglooValue,
    id::{DeviceID, EntityID, ExtensionID, GroupID},
    query::{DeviceSnapshot, EntitySnapshot, ExtensionSnapshot, GroupSnapshot},
    types::{agg::AggregationOp, compare::ComparisonOp, math::MathOp},
};
use bincode::{Decode, Encode};
use rustc_hash::FxHashSet;

// TODO if we make a Pyo3 rust python library for Extensions, we should
// be able to drop Borsh and just use Bincode. This way we can easily
// add more optimizations like SmallVecs
//
// By doing this we can potentially add huge optimizations. A complete
// Extension Rust library allows us to do really cool things like a A::SetFunction
// This would ship over the function to the Extension itself which spawns fake
// messages.
// For things like an RGB effect we can get serious performance benefits
// Alternatively, we can implement SetFunctions into the query engine.
// Regardless set functions have a huge benefit of:
//  1. Have penguin nodes for effects
//  2. Easily override/cancel effects - future Sets will stop the SetFunction

// FUTURE IDEAS:
//  4. temporal queries: values & agg (ex. mean of last month)

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum Query {
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
    ObserveAttached,

    Count,
    Inherit,
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

    ObserveName,
    /// device added, device removed
    ObserveMembership,

    Count,
    Inherit,
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
    ObserveAttached,
    ObserveName,

    Count,
    Inherit,
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
    Inherit,
    ObserveRegistered,
    ObserveComponentPut,
    // entities dont attach
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
    ObserveValue,

    Set(IglooValue),
    Put(IglooValue),
    Apply(MathOp),

    Count,
    Inherit,
}

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub enum EntityIDFilter {
    #[default]
    Any,
    Is(String),
    OneOf(FxHashSet<String>),
    /// glob pattern
    Matches(String),
}

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
    GroupMutation,

    DeviceID,
    DeviceSnapshot,
    DeviceAttached,
    DeviceRenamed,
    DeviceEntityAdded,

    EntitySnapshot,

    Aggregate(IglooType),
    ComponentValue(IglooType),
    ComponentValueWithParents(IglooType),

    Count,

    Observer(ObserverUpdateType),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ObserverUpdate {
    ExtensionAttached(ExtensionID, bool),

    GroupRenamed(GroupID, String),
    /// true => added, false => removed
    GroupMembershipChanged(GroupID, DeviceID, bool),

    DeviceRenamed(DeviceID, String),
    DeviceAttached(DeviceID, bool),

    EntityRegistered(DeviceID, EntityID),
    EntityComponentPut(DeviceID, EntityID, Component),

    Aggregate(IglooValue),
    ComponentValue(IglooValue),
    ComponentValueWithParents(DeviceID, EntityID, IglooValue),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ObserverUpdateType {
    ExtensionAttached,

    GroupRenamed,
    GroupMembership,

    DeviceRenamed,
    DeviceAttached,

    EntityRegistered,
    EntityComponentPut,

    Aggregate(IglooType),
    ComponentValue(IglooType),
    ComponentValueWithParents(IglooType),
}

impl Query {
    pub fn is_observer(&self) -> bool {
        match self {
            Query::Extension(q) => matches!(q.action, ExtensionAction::ObserveAttached),
            Query::Group(q) => matches!(
                q.action,
                GroupAction::ObserveName | GroupAction::ObserveMembership
            ),
            Query::Device(q) => matches!(
                q.action,
                DeviceAction::ObserveAttached | DeviceAction::ObserveName
            ),
            Query::Entity(q) => matches!(
                q.action,
                EntityAction::ObserveComponentPut | EntityAction::ObserveRegistered
            ),
            Query::Component(q) => matches!(q.action, ComponentAction::ObserveValue),
        }
    }
}
