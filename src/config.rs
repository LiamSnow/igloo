use std::{collections::HashMap, error::Error, fs};

use ron::{extensions::Extensions, Options};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct IglooConfig {
    pub version: f32,
    pub users: HashMap<String, User>,
    pub user_groups: HashMap<String, Vec<String>>,
    pub permissions: HashMap<String, String>,
    pub providers: Vec<Provider>,
    pub zones: IglooZones,
    pub automations: HashMap<String, Automation>
}

pub type IglooZones = HashMap<String, HashMap<String, DeviceConfig>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub password_hash: String,
    pub api_key_hash: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Provider {
    ESPHome(ESPHomeConfig),
    HomeKit(HomeKitConfig)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ESPHomeConfig {
    pub default_port: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HomeKitConfig {
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DeviceConfig {
    ESPHome(ESPHomeDeviceConfig),
    Test()
}

impl DeviceConfig {
    pub fn unwrap_esphome(self) -> ESPHomeDeviceConfig {
        if let Self::ESPHome(e) = self {
            return e
        }
        panic!("unwrap_esphome had wrong type!");
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ESPHomeDeviceConfig {
    pub ip: String,
    pub password: Option<String>,
    pub noise_psk: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Automation {
    pub trigger: AutomationTrigger,
    pub trigger_offset: Option<i32>,
    pub on_trigger: Vec<String>,
    pub on_change: Option<Vec<String>>
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AutomationTrigger {
    Button,
    Time(AutomationTimeTrigger),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutomationTimeTrigger {
    pub default: String,
}

impl IglooConfig {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        Self::parse(&fs::read_to_string(file_path)?)
    }

    pub fn parse(s: &str) -> Result<Self, Box<dyn Error>> {
        let options = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .with_default_extension(Extensions::UNWRAP_NEWTYPES)
            .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
        Ok(options.from_str(s)?)
    }
}
