use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProtocolConfig {
    pub commands: Vec<Command>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub id: u16,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub fields: Vec<CommandField>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandField {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub desc: String,
}

#[derive(Debug, Deserialize)]
pub struct ComponentsConfig {
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub name: String,
    pub id: u16,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub related: Vec<Related>,
    /// creates Supported{name}s
    /// a vector of the entire type
    #[serde(default)]
    pub derive_supported_type: Option<u16>,
    /// creates {name}List
    /// a vector of the entire type
    #[serde(default)]
    pub derive_list_type: Option<u16>,
    #[serde(flatten)]
    pub kind: ComponentKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ComponentKind {
    Single {
        field: String,
        /// creates {name}Min, {name}Max, {name}Step
        #[serde(default)]
        derive_bound_types: Option<[u16; 3]>,
        /// creates {name}List, which is a list
        /// of the INTERNAL type
        #[serde(default)]
        derive_inner_list_type: Option<u16>,
        /// creates {name}MinLength, {name}MaxLength, {name}Pattern
        /// only valid for String types
        #[serde(default)]
        derive_string_bound_types: Option<[u16; 3]>,
    },
    Struct {
        fields: Vec<Field>,
    },
    Enum {
        variants: Vec<Variant>,
    },
    Marker,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    // TODO add docs?
}

#[derive(Debug, Deserialize, Clone)]
pub struct Variant {
    pub id: u8,
    pub name: String,
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Related {
    pub name: String,
    pub reason: String,
}

impl ProtocolConfig {
    pub fn read(pathbuf: PathBuf) -> Self {
        let contents = fs::read_to_string(pathbuf).expect("Failed to read protocol file");
        toml::from_str(&contents).expect("Failed to parse protocol file")
    }
}

impl ComponentsConfig {
    pub fn read(pathbuf: PathBuf) -> Self {
        let contents = fs::read_to_string(pathbuf).expect("Failed to read components file");
        toml::from_str(&contents).expect("Failed to parse components file")
    }
}
