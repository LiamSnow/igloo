use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Component, Device};

/// MISO Floe sending command -> Igloo
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FloeCommand {
    AddDevice(Uuid, Device),
    ComponentUpdates(Vec<ComponentUpdate>),
    SaveConfig(String),
    /// NOT a lot, this is a response to a bad IglooCommand::Custom
    CustomError(String),
    Log(String),
}

/// MOSI Igloo sending command -> Floe
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IglooCommand {
    /// Igloo sends this at boot
    Init(InitPayload),
    /// request update is just asking for this to be done
    /// it is only confirmed to the device tree once the
    /// Floe sends back an Update
    ReqComponentUpdates(Vec<ComponentUpdate>),
    /// command_name, payload
    Custom(String, String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitPayload {
    pub config: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentUpdate {
    pub device: Uuid,
    pub entity: String,
    pub value: Component,
}
