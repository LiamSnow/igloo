use crate::{
    ComponentType, IglooType, IglooValue,
    id::{DeviceID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
    query::{DeviceGroupFilter, EntityIDFilter, IDFilter, TypeFilter},
    types::agg::AggregationOp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchQuery {
    Metadata,
    Component(WatchComponentQuery),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WatchComponentQuery {
    #[serde(default)]
    pub device_filter: WatchDeviceFilter,
    #[serde(default)]
    pub entity_filter: WatchEntityFilter,
    pub component: ComponentType,
    #[serde(default)]
    pub post_op: Option<AggregationOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchDeviceFilter {
    pub id: IDFilter<DeviceID>,
    pub owner: IDFilter<ExtensionID>,
    pub group: DeviceGroupFilter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchEntityFilter {
    pub id: EntityIDFilter,
    pub type_filter: Option<TypeFilter>,
}

// -- Responses

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WatchUpdateType {
    Metadata,
    ComponentAggregate(IglooType),
    ComponentValue(IglooType),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WatchUpdate {
    Metadata(Vec<MetadataUpdate>),
    ComponentAggregate(IglooValue),
    ComponentValue(DeviceID, EntityIndex, IglooValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceMetadata {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupMetadata {
    pub name: String,
    pub devices: Vec<DeviceID>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    pub index: ExtensionIndex,
    pub devices: Vec<DeviceID>,
}
