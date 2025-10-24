use crate::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct WiringData {
    pub start_node: NodeID,
    pub start_pin: PinID,
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
