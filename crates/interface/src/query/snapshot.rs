use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::generated::{
    component::Component,
    id::{DeviceId, EntityId, EntityIndex, ExtensionId, ExtensionIndex, GroupId},
};

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("{id}{{name={name},devices=[..]}}")]
pub struct GroupSnapshot {
    pub id: GroupId,
    pub name: String,
    pub devices: Vec<DeviceId>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("{id}{{index={index}}}")]
pub struct ExtensionSnapshot {
    pub id: ExtensionId,
    pub index: ExtensionIndex,
    pub devices: Vec<DeviceId>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("{id}{{name={name},owner={owner},owner_ref={owner_ref:?},entities=[..],groups=[..]}}")]
pub struct DeviceSnapshot {
    pub id: DeviceId,
    pub name: String,
    pub entities: Vec<EntitySnapshot>,
    pub owner: ExtensionId,
    pub owner_ref: Option<ExtensionIndex>,
    pub groups: Vec<GroupId>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("Entity{{id={id},index={index},components=[..]}}")]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub index: EntityIndex,
    pub components: Vec<Component>,
    pub parent: DeviceId,
}
