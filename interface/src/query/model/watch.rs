use crate::{
    ComponentType, IglooType, IglooValue,
    id::{DeviceID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
    query::{DeviceGroupFilter, EntityIDFilter, IDFilter, TypeFilter},
    types::agg::AggregationOp,
};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WatchQuery {
    Metadata,
    Component(WatchComponentQuery),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WatchComponentQuery {
    #[cfg_attr(feature = "serde", serde(default))]
    pub device_filter: WatchDeviceFilter,
    #[cfg_attr(feature = "serde", serde(default))]
    pub entity_filter: WatchEntityFilter,
    pub component: ComponentType,
    #[cfg_attr(feature = "serde", serde(default))]
    pub post_op: Option<AggregationOp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct WatchDeviceFilter {
    pub id: IDFilter<DeviceID>,
    pub owner: IDFilter<ExtensionID>,
    pub group: DeviceGroupFilter,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct WatchEntityFilter {
    pub id: EntityIDFilter,
    pub type_filter: Option<TypeFilter>,
}

// -- Responses

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WatchUpdateType {
    Metadata,
    ComponentAggregate(IglooType),
    ComponentValue(IglooType),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WatchUpdate {
    Metadata(Vec<MetadataUpdate>),
    ComponentAggregate(IglooValue),
    ComponentValue(DeviceID, EntityIndex, IglooValue),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceMetadata {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupMetadata {
    pub name: String,
    pub devices: Vec<DeviceID>,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtensionMetadata {
    pub index: ExtensionIndex,
    pub devices: Vec<DeviceID>,
}
