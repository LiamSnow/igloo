use crate::{
    ComponentType, IglooType, IglooValue,
    id::{DeviceID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
    query::{DeviceGroupFilter, EntityIDFilter, IDFilter, TypeFilter},
    types::agg::AggregationOp,
};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum WatchQuery {
    Metadata,
    Component(WatchComponentQuery),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct WatchComponentQuery {
    pub device_id: IDFilter<DeviceID>,
    pub entity_id: EntityIDFilter,
    pub owner: IDFilter<ExtensionID>,
    pub group: DeviceGroupFilter,

    pub type_filter: Option<TypeFilter>,
    pub component: ComponentType,

    pub post_op: Option<AggregationOp>,
}

// -- Responses

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum WatchUpdateType {
    Metadata,
    ComponentAggregate(IglooType),
    ComponentValue(IglooType),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum WatchUpdate {
    Metadata(Vec<MetadataUpdate>),
    ComponentAggregate(IglooValue),
    ComponentValue(DeviceID, EntityIndex, IglooValue),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum MetadataUpdate {
    /// device added or changed
    Device(DeviceID, DeviceMetadata),
    DeviceRemoved(DeviceID),

    /// group added or changed
    Group(GroupID, GroupMetadata),
    GroupRemoved(GroupID),

    /// extension added or changed
    Extension(ExtensionID, ExtensionMetadata),
    ExtensionRemoved(ExtensionID),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct DeviceMetadata {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct GroupMetadata {
    pub name: String,
    pub devices: Vec<DeviceID>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct ExtensionMetadata {
    pub index: ExtensionIndex,
    pub devices: Vec<DeviceID>,
}
