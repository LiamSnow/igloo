use std::{collections::HashMap, fs};

use ron::{extensions::Extensions, Options};
use serde::{Deserialize, Serialize};

use crate::{
    entity::EntityType,
    providers::{DeviceConfig, ProviderConfig},
    scripts::{ScriptClaims, ScriptMeta},
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
    pub fn from_file(file_path: &str) -> Self {
        let file_res = fs::read_to_string(file_path);
        if let Err(e) = file_res {
            panic!("Failed reading config file: {e}")
        }

        Self::parse(file_res.unwrap())
    }

    pub fn parse(s: String) -> Self {
        let options = Options::default()
            .with_default_extension(Extensions::IMPLICIT_SOME)
            .with_default_extension(Extensions::UNWRAP_NEWTYPES)
            .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES);
        match options.from_str(&s) {
            Ok(r) => r,
            Err(e) => {
                panic!(
                    "Failed parsing config file:\n {} at {}",
                    e.code, e.position
                );
            }
        }
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
    Script(String),

    BasicLight(String),
    CTLight(String),
    RGBLight(String),
    RGBCTLight(String),
    Bool(String),
    Time(String),
    DateTime(String),
    Weekly(String),
    Int(String),
    Float(String),
    Text(String),
}

impl UIElementConfig {
    /// returns (selection, entity_type) if applicable
    pub fn get_meta(&self) -> Option<(&str, EntityType)> {
        match self {
            Self::BasicLight(s) => Some((s, EntityType::Light)),
            Self::CTLight(s) => Some((s, EntityType::Light)),
            Self::RGBLight(s) => Some((s, EntityType::Light)),
            Self::RGBCTLight(s) => Some((s, EntityType::Light)),
            Self::Bool(s) => Some((s, EntityType::Bool)),
            Self::Time(s) => Some((s, EntityType::Time)),
            Self::DateTime(s) => Some((s, EntityType::DateTime)),
            Self::Weekly(s) => Some((s, EntityType::Weekly)),
            Self::Int(s) => Some((s, EntityType::Int)),
            Self::Float(s) => Some((s, EntityType::Float)),
            Self::Text(s) => Some((s, EntityType::Text)),
            Self::Button(..) | Self::Script(..) => None,
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

fn get_false() -> bool {
    false
}
fn get_true() -> bool {
    false
}

#[derive(Deserialize, Clone)]
pub enum BasicScriptLine {
    Command(String),
    HttpGet { url: String },
    HttpPost { url: String, body: String },
    Set(usize, String),
    Delay(u64),
    Forever(Vec<BasicScriptLine>),
}
