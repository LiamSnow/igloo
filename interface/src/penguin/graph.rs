use std::collections::HashMap;

use crate::penguin::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct PenguinGraph {
    pub nodes: HashMap<NodeID, Node>,
    pub wires: HashMap<WireID, Wire>,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Debug,
    Default,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct NodeID(pub u16);

#[derive(Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct Node {
    pub defn_ref: NodeDefnRef,
    pub x: f64,
    pub y: f64,
    /// values for nodes with NodeConfig::Input
    pub inputs: HashMap<InputID, PenguinValue>,
    /// values of pins which are unconnected
    pub values: HashMap<PinID, PenguinValue>,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Default,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct WireID(pub u16);

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Wire {
    pub from_node: NodeID,
    pub from_pin: PinID,
    pub to_node: NodeID,
    pub to_pin: PinID,
    pub r#type: PinType,
}
