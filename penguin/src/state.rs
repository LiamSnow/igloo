use igloo_interface::{NodeDefn, PenguinRegistry, PinType};

use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct WiringData {
    pub start_node: NodeID,
    pub start_pin: PinRef,
    pub is_output: bool,
    pub wire_type: PinType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridSettings {
    pub enabled: bool,
    pub snap: bool,
    pub size: f64,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap: true,
            size: 20.0,
        }
    }
}

impl WiringData {
    pub fn is_valid_end(&self, end_node: NodeID, end_type: PinType, end_is_out: bool) -> bool {
        let compatible = if self.is_output {
            self.wire_type.can_connect_to(end_type)
        } else {
            end_type.can_connect_to(self.wire_type)
        };

        self.is_output != end_is_out && compatible && self.start_node != end_node
    }

    pub fn cast_name(&self, end_type: PinType) -> Option<String> {
        if self.is_output {
            self.wire_type.cast_name(end_type)
        } else {
            end_type.cast_name(self.wire_type)
        }
    }

    /// Convert starting and ending pins to from (output) and to (input)
    pub fn resolve_connection(
        &self,
        end_node: NodeID,
        end_pin: PinRef,
    ) -> (NodeID, PinRef, NodeID, PinRef) {
        if self.is_output {
            (self.start_node, self.start_pin, end_node, end_pin)
        } else {
            (end_node, end_pin, self.start_node, self.start_pin)
        }
    }

    /// Find all node defs with compatible inputs
    /// Returns Iter<(library_name, node_name, node_defn)>
    pub fn find_compatible_nodes<'a>(
        &self,
        registry: &'a PenguinRegistry,
    ) -> impl Iterator<Item = (&'a String, &'a String, &'a NodeDefn)> {
        let wire_type = self.wire_type;
        registry.libraries.values().flat_map(move |lib| {
            lib.nodes
                .iter()
                .filter(move |(_, defn)| defn.has_compatible_input(wire_type))
                .map(move |(name, defn)| (&lib.name, name, defn))
        })
    }
}
