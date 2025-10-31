use std::collections::HashMap;

use crate::penguin::*;
use borsh::{BorshDeserialize, BorshSerialize};

/// PenguinGraph is meant to be a reliable serialization format
/// for saving and transferring around graphs.
/// It is not intended to be used for traversing the graph, instead
/// users of this should transform it into a different structure.
/// I have chosen to do it this way because the UI and Server
/// have very different requirements for what they need to do
/// with the graph.
#[derive(Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct PenguinGraph {
    pub nodes: HashMap<PenguinNodeID, PenguinNode>,
    pub wires: HashMap<PenguinWireID, PenguinWire>,
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
pub struct PenguinNodeID(pub u16);

#[derive(Debug, Default, Clone, BorshSerialize, BorshDeserialize)]
pub struct PenguinNode {
    pub defn_ref: PenguinNodeDefnRef,
    pub x: f64,
    pub y: f64,
    /// values for nodes with NodeConfig::Input
    pub input_cfg_values: HashMap<InputID, PenguinValue>,
    /// values of pins which are unconnected
    pub input_pin_values: HashMap<PenguinPinID, PenguinValue>,
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
pub struct PenguinWireID(pub u16);

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct PenguinWire {
    pub from_node: PenguinNodeID,
    pub from_pin: PenguinPinID,
    pub to_node: PenguinNodeID,
    pub to_pin: PenguinPinID,
    pub r#type: PenguinPinType,
}

impl PenguinNode {
    pub fn new(defn_ref: PenguinNodeDefnRef, x: f64, y: f64) -> Self {
        Self {
            defn_ref,
            x,
            y,
            ..Default::default()
        }
    }
}
