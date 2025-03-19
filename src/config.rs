use std::{collections::HashMap, error::Error, fs};

use ron::{extensions::Extensions, Options};
use serde::{Deserialize, Serialize};

use crate::{
    providers::{DeviceConfig, ProviderConfig},
    scripts::{ScriptClaims, ScriptMeta},
    subdevice::SubdeviceType,
};

#[derive(Deserialize)]
pub struct IglooConfig {
    pub version: f32,
    pub auth: AuthConfig,
    pub providers: Vec<ProviderConfig>,
    pub devices: DeviceConfigs,
    pub ui: Vec<(String, Vec<UIElementConfig>)>,
    pub scripts: HashMap<String, ScriptConfig>,
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

pub type DeviceConfigs = HashMap<String, HashMap<String, DeviceConfig>>;

#[derive(Deserialize)]
pub struct AuthConfig {
    pub users: HashMap<String, UserConfig>,
    pub groups: HashMap<String, Vec<String>>,
    pub permissions: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    pub password_hash: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum UIElementConfig {
    Button(String, String),
    BasicLight(String),
    CTLight(String),
    RGBLight(String),
    RGBCTLight(String),
    Switch(String),
    Time(String),
    Int(String),
}

impl UIElementConfig {
    /// returns (selection, subdev_type) if applicable
    pub fn get_meta(&self) -> Option<(&str, SubdeviceType)> {
        match self {
            Self::BasicLight(s) => Some((s, SubdeviceType::Light)),
            Self::CTLight(s) => Some((s, SubdeviceType::Light)),
            Self::RGBLight(s) => Some((s, SubdeviceType::Light)),
            Self::RGBCTLight(s) => Some((s, SubdeviceType::Light)),
            Self::Switch(s) => Some((s, SubdeviceType::Switch)),
            Self::Time(s) => Some((s, SubdeviceType::Time)),
            Self::Int(s) => Some((s, SubdeviceType::Int)),
            _ => None,
        }
    }

    pub fn get_command(&self) -> Option<&str> {
        Some(match self {
            Self::Button(_name, cmd) => cmd,
            _ => return None,
        })
    }
}

#[derive(Deserialize)]
pub enum ScriptConfig {
    Python(PythonScriptConfig),
    Basic(BasicScriptConfig),
}

impl ScriptConfig {
    /// returns claims, auto_cancel, auto_run
    pub fn get_meta(&self) -> ScriptMeta {
        match self {
            ScriptConfig::Python(cfg) => ScriptMeta {
                claims: &cfg.claims,
                auto_cancel: cfg.auto_cancel,
                auto_run: cfg.auto_run,
            },
            ScriptConfig::Basic(cfg) => ScriptMeta {
                claims: &cfg.claims,
                auto_cancel: cfg.auto_cancel,
                auto_run: cfg.auto_run,
            },
        }
    }
}

#[derive(Deserialize)]
pub struct PythonScriptConfig {
    #[serde(default)]
    pub claims: ScriptClaims,
    #[serde(default = "get_true")]
    pub auto_cancel: bool,
    #[serde(default = "get_false")]
    pub auto_run: bool,
    pub file: String,
}

#[derive(Deserialize)]
pub struct BasicScriptConfig {
    #[serde(default)]
    pub claims: ScriptClaims,
    #[serde(default = "get_true")]
    pub auto_cancel: bool,
    #[serde(default = "get_false")]
    pub auto_run: bool,
    pub body: Vec<BasicScriptLine>,
}

fn get_false() -> bool { false }
fn get_true() -> bool { false }

#[derive(Deserialize, Clone)]
pub enum BasicScriptLine {
    Command(String),
    HttpGet { url: String },
    HttpPost { url: String, body: String },
    Save(usize, String),
    Delay(u64),
    Forever(Vec<BasicScriptLine>),
}
