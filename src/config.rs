use std::{collections::HashMap, error::Error, fs};

use chrono::NaiveTime;
use ron::{extensions::Extensions, Options};
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    command::SubdeviceType,
    providers::{DeviceConfig, ProviderConfig},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct IglooConfig {
    pub version: f32,
    pub users: HashMap<String, User>,
    pub user_groups: HashMap<String, Vec<String>>,
    pub permissions: HashMap<String, String>,
    pub providers: Vec<ProviderConfig>,
    pub zones: ZonesConfig,
    pub ui: HashMap<String, Vec<UIElementConfig>>,
    pub automations: HashMap<String, Automation>,
}

pub type ZonesConfig = HashMap<String, HashMap<String, DeviceConfig>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub password_hash: String,
    pub api_key_hash: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
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

    pub fn get_default_value(&self) -> Option<ElementValue> {
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
}

#[derive(Debug, Serialize, Clone)]
pub enum ElementValue {
    #[serde(serialize_with = "serialize_time")]
    Time(NaiveTime),
}

pub fn serialize_time<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&time.format("%H:%M").to_string())
}

pub fn parse_time(time_str: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(&time_str, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(&time_str, "%I:%M %p"))
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LightFeature {
    RGB,
    CT,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ButtonConfig {
    name: String,
    on_click: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeSelectorConfig {
    name: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)] //FIXME
    default: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Automation {
    Time(TimeAutomation),
    None(NoneAutomation),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NoneAutomation {
    pub on_trigger: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeAutomation {
    pub selector: String,
    pub trigger_offset: Option<i32>,
    pub on_trigger: Vec<String>,
    pub on_change: Option<Vec<String>>,
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
