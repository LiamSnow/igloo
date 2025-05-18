pub mod client;
pub mod broker;
pub use client::task;
pub use broker::init_provider;

use crate::entity::EntityType;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
   brokers: Vec<MQTTBrokerConfig>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MQTTBrokerConfig {
    port: u16
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    /// entity_name -> mqtt path
    /// creates `text` entities
    publish: HashMap<String, PublishType>,
    /// mqtt path -> cmd
    subscribe: HashMap<String, ResponsePlan>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PublishType {
    Trigger(String),
    Text(String),
    Bool(String),
    Int(String),
    Float(String),
}

impl PublishType {
    pub fn get_entity_type(&self) -> EntityType {
        match self {
            Self::Trigger(_) => EntityType::Trigger,
            Self::Text(_) => EntityType::Text,
            Self::Bool(_) => EntityType::Bool,
            Self::Int(_) => EntityType::Int,
            Self::Float(_) => EntityType::Float,
        }
    }

    pub fn get_path(&self) -> &str {
        match self {
            Self::Trigger(s) => s,
            Self::Text(s) => s,
            Self::Bool(s) => s,
            Self::Int(s) => s,
            Self::Float(s) => s,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResponsePlan {
    /// response, dest_path
    RespondWith(String, String),
    /// cmd, dest_path
    RespondWithCommandResult(String, String),
    RunCommand(String),
    // SaveState()
}
