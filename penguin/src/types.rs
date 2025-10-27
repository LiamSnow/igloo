use dioxus::prelude::*;
use euclid::default::Point2D;
use igloo_interface::{NodeDefn, NodeDefnRef, PinDefn, PinDefnType, PinType, ValueData, ValueType};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct NodeID(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct WireID(pub u16);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PinRef {
    pub defn_index: usize,
    pub phantom_instance: usize,
}

#[derive(Debug, Store, PartialEq, Clone, Default)]
pub struct Node {
    pub defn_ref: NodeDefnRef,
    pub pos: Point2D<f64>,
    /// num pins for each PinDefnType::Phantom
    pub phantom_state: HashMap<u8, usize>,
    /// values for pins with PinDefnType::DynValue
    pub dynvalue_state: HashMap<u8, ValueType>,
    /// values for nodes with NodeConfig::Input
    pub value_inputs: HashMap<u8, ValueData>,
    /// values of pins which are unconnected
    pub pin_values: HashMap<PinRef, ValueData>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wire {
    pub from_node: NodeID,
    pub from_pin: PinRef,
    pub to_node: NodeID,
    pub to_pin: PinRef,
    pub r#type: PinType,
}

impl Node {
    fn get_phantom_min(&self, defn: &igloo_interface::NodeDefn, phantom_id: u8) -> usize {
        defn.cfg
            .iter()
            .find_map(|cfg| {
                if let igloo_interface::NodeConfig::AddRemovePin(config) = cfg {
                    if config.phantom_id == phantom_id {
                        return Some(config.min as usize);
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
                        PinRef {
                            defn_index,
                            phantom_instance: 0,
                        },
                        PinType::Flow,
                        pin_defn.name.clone(),
                    ));
                }
                PinDefnType::Value(vt) => {
                    result.push((
                        PinRef {
                            defn_index,
                            phantom_instance: 0,
                        },
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
                            PinRef {
                                defn_index,
                                phantom_instance: i,
                            },
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
                        PinRef {
                            defn_index,
                            phantom_instance: 0,
                        },
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
    pub fn new(defn_index: usize) -> Self {
        Self {
            defn_index,
            phantom_instance: 0,
        }
    }

    pub fn with_phantom_inst(defn_index: usize, phantom_instance: usize) -> Self {
        Self {
            defn_index,
            phantom_instance,
        }
    }
}
