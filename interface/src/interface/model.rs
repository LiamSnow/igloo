use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{ComponentValue, Device};

/// Floe sending command -> Igloo
#[derive(Serialize, Deserialize, Clone)]
pub enum FloeCommand {
    AddDevice(Uuid, Device),
    Update(Update),
    SaveConfig(String),
    Log(String),
}

/// Igloo sending command -> Floe
#[derive(Serialize, Deserialize, Clone)]
pub enum IglooCommand {
    Update(Update),
    Config(String),
    Ping,
    Custom(String, String),
}

pub type IglooResponse = Result<(), IglooResponseError>;
pub type FloeResponse = Result<Option<String>, String>;

#[derive(Serialize, Deserialize, Clone, Error, Debug)]
pub enum IglooResponseError {
    #[error("Device `{0}` does not exist!")]
    InvalidDevice(Uuid),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Update {
    pub device: Uuid,
    pub entity: String,
    pub value: ComponentValue,
}
