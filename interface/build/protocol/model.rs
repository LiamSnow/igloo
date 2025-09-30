use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProtocolConfig {
    pub version: u8,
    pub commands: Commands,
}

#[derive(Debug, Deserialize)]
pub struct Commands {
    /// MISO Floe sending command -> Igloo
    pub floe: Vec<Command>,
    /// MOSI Igloo sending command -> Floe
    pub igloo: Vec<Command>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub opcode: u8,
    pub desc: String,
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
