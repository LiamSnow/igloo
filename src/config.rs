use std::{collections::HashMap, error::Error, fs};

use ron::{extensions::Extensions, Options};
use serde::{Deserialize, Serialize};

use crate::{
    command::SubdeviceType, elements::{parse_time, ElementValue}, providers::{DeviceConfig, ProviderConfig}
};

#[derive(Debug, Deserialize, Serialize)]
pub struct IglooConfig {
    pub version: f32,
    pub users: HashMap<String, User>,
    pub user_groups: HashMap<String, Vec<String>>,
    pub permissions: HashMap<String, String>,
    pub providers: Vec<ProviderConfig>,
    pub zones: ZonesConfig,
    pub ui: Vec<(String, Vec<UIElementConfig>)>,
    pub scripts: HashMap<String, Vec<String>>,
}

pub type ZonesConfig = HashMap<String, HashMap<String, DeviceConfig>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum UIElementConfig {
    BasicLight(String),
    CTLight(String),
    RGBLight(String),
    RGBCTLight(String),
    Switch(String),
    Button(ButtonConfig),
    TimeSelector(TimeSelectorConfig),
}

impl UIElementConfig {
    pub fn get_sel_and_subdev_type(&self) -> Option<(&str, SubdeviceType)> {
        match self {
            Self::BasicLight(s) => Some((s, SubdeviceType::Light)),
            Self::CTLight(s) => Some((s, SubdeviceType::Light)),
            Self::RGBLight(s) => Some((s, SubdeviceType::Light)),
            Self::RGBCTLight(s) => Some((s, SubdeviceType::Light)),
            Self::Switch(s) => Some((s, SubdeviceType::Switch)),
            _ => None,
        }
    }

    pub fn get_def_val(&self) -> Option<ElementValue> {
        Some(match self {
            Self::TimeSelector(ref cfg) => {
                ElementValue::Time(parse_time(&cfg.default).unwrap()) //FIXME
            }
            _ => return None,
        })
    }

    pub fn get_name(&self) -> Option<&str> {
        Some(match self {
            Self::Button(c) => &c.name,
            Self::TimeSelector(c) => &c.name,
            _ => return None,
        })
    }

    pub fn get_commands(&self) -> Option<Vec<&str>> {
        Some(match self {
            Self::Button(c) => vec![&c.on_click],
            _ => return None,
        })
    }

    pub fn get_scripts(&self) -> Option<Vec<&str>> {
        Some(match self {
            Self::TimeSelector(c) => {
                let mut v = Vec::new();
                if let Some(s) = &c.on_trigger {
                    v.push(s.as_str());
                }
                if let Some(s) = &c.on_change {
                    v.push(s.as_str());
                }
                v
            },
            _ => return None,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LightFeature {
    RGB,
    CT,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ButtonConfig {
    name: String,
    on_click: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TimeSelectorConfig {
    name: String,
    #[serde(skip_serializing)]
    default: String,
    #[serde(skip_serializing)]
    pub trigger_offset: Option<i32>,
    #[serde(skip_serializing)]
    pub on_trigger: Option<String>,
    #[serde(skip_serializing)]
    pub on_change: Option<String>,
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
