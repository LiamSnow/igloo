use crate::Component;
use serde::{Deserialize, Serialize};

pub const DATA_PATH_ENV_VAR: &str = "DATA_PATH";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtensionToIgloo {
    /// Initiates communication
    /// Extension must send on boot
    WhatsUpIgloo,

    // UpgradeTo { version: u16 }
    CreateDevice {
        name: String,
    },

    RegisterEntity {
        device: u64,
        entity_id: String,
        entity_index: usize,
    },

    WriteComponents {
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IglooToExtension {
    DeviceCreated {
        name: String,
        id: u64,
    },

    WriteComponents {
        device: u64,
        entity: usize,
        comps: Vec<Component>,
    },

    Custom {
        name: String,
        payload: serde_json::Value,
    },
}
