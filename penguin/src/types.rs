use borsh::{BorshDeserialize, BorshSerialize};
use dioxus::prelude::*;
use euclid::default::Point2D;
use igloo_interface::{NodeDefn, NodeDefnRef, PinDefn, PinDefnType, PinType, ValueData, ValueType};
use std::collections::HashMap;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, BorshSerialize, BorshDeserialize)]
pub struct WireID(pub u16);

#[derive(
    Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Debug, BorshSerialize, BorshDeserialize,
)]
pub struct PinRef {
    pub defn_index: u32,
    pub phantom_instance: u8,
}

#[derive(Debug, Store, PartialEq, Clone, Default, BorshSerialize, BorshDeserialize)]
pub struct Node {
    pub defn_ref: NodeDefnRef,
    pub x: f64,
    pub y: f64,
    /// num pins for each PinDefnType::Phantom
    pub phantom_state: HashMap<u8, u8>,
    /// values for pins with PinDefnType::DynValue
    pub dynvalue_state: HashMap<u8, ValueType>,
    /// values for nodes with NodeConfig::Input
    pub value_inputs: HashMap<u8, ValueData>,
    /// values of pins which are unconnected
    pub pin_values: HashMap<PinRef, ValueData>,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Wire {
    pub from_node: NodeID,
    pub from_pin: PinRef,
    pub to_node: NodeID,
    pub to_pin: PinRef,
    pub r#type: PinType,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct PenguinClipboard {
    pub nodes: HashMap<NodeID, Node>,
    pub wires: Vec<Wire>,
}

impl Node {
    pub fn pos(&self) -> Point2D<f64> {
        Point2D::new(self.x, self.y)
    }

    fn get_phantom_min(&self, defn: &igloo_interface::NodeDefn, phantom_id: u8) -> u8 {
        defn.cfg
            .iter()
            .find_map(|cfg| {
                if let igloo_interface::NodeConfig::AddRemovePin(config) = cfg {
                    if config.phantom_id == phantom_id {
                        return Some(config.min);
                    }
                }
                None
            })
            .unwrap_or(0)
    }

    fn get_phantom_type(&self, defn: &igloo_interface::NodeDefn, phantom_id: u8) -> PinType {
        defn.cfg
            .iter()
            .find_map(|cfg| {
                if let igloo_interface::NodeConfig::AddRemovePin(config) = cfg {
                    if config.phantom_id == phantom_id {
                        return Some(config.r#type);
                    }
                }
                None
            })
            .unwrap_or(PinType::Flow)
    }

    fn expand_pins(&self, defn: &NodeDefn, pins: &[PinDefn]) -> Vec<(PinRef, PinType, String)> {
        let mut result = Vec::new();

        for (defn_index, pin_defn) in pins.iter().enumerate() {
            match &pin_defn.r#type {
                PinDefnType::Flow => {
                    result.push((
                        PinRef::new(defn_index as u32),
                        PinType::Flow,
                        pin_defn.name.clone(),
                    ));
                }
                PinDefnType::Value(vt) => {
                    result.push((
                        PinRef::new(defn_index as u32),
                        PinType::Value(*vt),
                        pin_defn.name.clone(),
                    ));
                }
                PinDefnType::Phantom(phantom_id) => {
                    let count = self
                        .phantom_state
                        .get(phantom_id)
                        .copied()
                        .unwrap_or_else(|| self.get_phantom_min(defn, *phantom_id));

                    let pin_type = self.get_phantom_type(defn, *phantom_id);

                    for i in 0..count {
                        result.push((
                            PinRef::with_phantom_inst(defn_index as u32, i as u8),
                            pin_type,
                            pin_defn.name.clone(),
                        ));
                    }
                }
                PinDefnType::DynValue(dyn_id) => {
                    let value_type = self
                        .dynvalue_state
                        .get(dyn_id)
                        .copied()
                        .unwrap_or(ValueType::Int);
                    result.push((
                        PinRef::new(defn_index as u32),
                        PinType::Value(value_type),
                        pin_defn.name.clone(),
                    ));
                }
            }
        }

        result
    }

    pub fn expand_inputs(
        &self,
        defn: &igloo_interface::NodeDefn,
    ) -> Vec<(PinRef, PinType, String)> {
        self.expand_pins(defn, &defn.inputs)
    }

    pub fn expand_outputs(
        &self,
        defn: &igloo_interface::NodeDefn,
    ) -> Vec<(PinRef, PinType, String)> {
        self.expand_pins(defn, &defn.outputs)
    }
}

impl Wire {
    pub fn connects_to(&self, node: NodeID, pin: &PinRef, is_output: bool) -> bool {
        if is_output {
            self.from_node == node && &self.from_pin == pin
        } else {
            self.to_node == node && &self.to_pin == pin
        }
    }
}

impl PinRef {
    pub fn new(defn_index: u32) -> Self {
        Self {
            defn_index,
            phantom_instance: 0,
        }
    }

    pub fn with_phantom_inst(defn_index: u32, phantom_instance: u8) -> Self {
        Self {
            defn_index,
            phantom_instance,
        }
    }
}
