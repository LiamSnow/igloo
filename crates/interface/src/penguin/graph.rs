use crate::{
    penguin::*,
    types::{IglooType, IglooValue},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// PenguinGraph is meant to be a reliable serialization format
/// for saving and transferring around graphs.
/// It is not intended to be used for traversing the graph, instead
/// users of this should transform it into a different structure.
/// I have chosen to do it this way because the UI and Server
/// have very different requirements for what they need to do
/// with the graph.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PenguinGraph {
    pub nodes: HashMap<PenguinNodeID, PenguinNode>,
    pub wires: HashMap<PenguinWireID, PenguinWire>,
}

#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default, Serialize, Deserialize,
)]
pub struct PenguinNodeID(pub u16);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PenguinNode {
    pub defn_ref: PenguinNodeDefnRef,
    pub x: f64,
    pub y: f64,
    /// values for each NodeFeature::Input
    pub input_feature_values: HashMap<NodeInputFeatureID, PenguinInputValue>,
    // TODO also track size of textareas
    /// values of pins which are unconnected
    pub input_pin_values: HashMap<PenguinPinID, PenguinInputValue>,
    /// only for resizable nodes (currently only sections)
    pub size: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenguinInputValue {
    pub value: IglooValue,
    pub size: Option<(i32, i32)>,
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct PenguinWireID(pub u16);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

    pub fn ensure_input_feature_value(&mut self, feature: &NodeInputFeature) {
        if !self.input_feature_values.contains_key(&feature.id) {
            self.input_feature_values.insert(
                feature.id.clone(),
                PenguinInputValue::new(IglooValue::default(&feature.value_type)),
            );
        }
    }

    pub fn ensure_input_pin_value(&mut self, pin_id: &PenguinPinID, r#type: &IglooType) {
        if !self.input_pin_values.contains_key(pin_id) {
            self.input_pin_values.insert(
                pin_id.clone(),
                PenguinInputValue::new(IglooValue::default(r#type)),
            );
        }
    }
}

impl PenguinInputValue {
    pub fn new(value: IglooValue) -> Self {
        Self {
            size: match &value {
                IglooValue::Text(_) => Some((100, 20)),
                _ => None,
            },
            value,
        }
    }

    pub fn set_from_string(&mut self, value: String) -> bool {
        if let Some(new) = IglooValue::from_string(&self.value.r#type(), value) {
            self.value = new;
            true
        } else {
            false
        }
    }
}
