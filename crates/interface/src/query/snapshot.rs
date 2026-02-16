use crate::{
    Component,
    id::{DeviceID, EntityID, EntityIndex, ExtensionID, ExtensionIndex, GroupID},
};
use derive_more::Display;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("{id}{{name={name},devices=[..]}}")]
pub struct GroupSnapshot {
    pub id: GroupID,
    pub name: String,
    pub devices: FxHashSet<DeviceID>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("{id}{{index={index}}}")]
pub struct ExtensionSnapshot {
    pub id: ExtensionID,
    pub index: ExtensionIndex,
    pub devices: Vec<DeviceID>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("{id}{{name={name},owner={owner},owner_ref={owner_ref:?},entities=[..],groups=[..]}}")]
pub struct DeviceSnapshot {
    pub id: DeviceID,
    pub name: String,
    pub entities: Vec<EntitySnapshot>,
    pub owner: ExtensionID,
    pub owner_ref: Option<ExtensionIndex>,
    pub groups: FxHashSet<GroupID>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, Serialize, Deserialize)]
#[display("Entity{{id={id},index={index},components=[..]}}")]
pub struct EntitySnapshot {
    pub id: EntityID,
    pub index: EntityIndex,
    pub components: Vec<Component>,
    pub parent: DeviceID,
}
