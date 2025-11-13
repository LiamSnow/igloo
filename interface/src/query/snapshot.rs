use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::Display;

use crate::{
    Component,
    id::{DeviceID, FloeID, FloeRef, GroupID},
};

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("{id}{{name={name},devices=[..]}}")]
pub struct GroupSnapshot {
    pub id: GroupID,
    pub name: String,
    pub devices: Vec<DeviceID>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("{id}{{ref={fref},msc={max_supported_component}}}")]
pub struct FloeSnapshot {
    pub id: FloeID,
    pub fref: FloeRef,
    pub max_supported_component: u16,
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("{id}{{name={name},owner={owner},entities=[..],groups=[..]}}")]
pub struct DeviceSnapshot {
    pub id: DeviceID,
    pub name: String,
    pub entities: Vec<EntitySnapshot>,
    pub owner: FloeID,
    pub groups: Vec<GroupID>,
}

#[derive(Debug, Clone, PartialEq, Display, Default, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
#[display("Entity{{name={name},parent={parent},components=[..]}}")]
pub struct EntitySnapshot {
    pub name: String,
    pub components: Vec<Component>,
    pub parent: DeviceID,
}
