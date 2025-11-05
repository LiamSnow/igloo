use std::{fs, path::PathBuf};

use serde::Deserialize;

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
    #[serde(flatten)]
    #[serde(default)]
    pub kind: ComponentKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ComponentKind {
    Enum {
        kind: EnumTag,
        variants: Vec<Variant>,
    },
    Single {
        kind: IglooType,
    },
    Marker {
        #[serde(default)]
        kind: Option<MarkerTag>,
    },
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Marker {
            kind: Some(MarkerTag::Marker),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum EnumTag {
    Enum,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MarkerTag {
    Marker,
}

#[derive(Debug, Deserialize, Clone)]
pub enum IglooType {
    Integer,
    Real,
    Text,
    Boolean,
    Color,
    Date,
    Time,
    IntegerList,
    RealList,
    TextList,
    BooleanList,
    ColorList,
    DateList,
    TimeList,
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
