use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceConfig {
    pub r#type: VarType,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum VarType {
    Int {
        range: Option<(i32, i32)>,
        default: i32,
    },
    Float {
        range: Option<(f32, f32)>,
        default: f32,
    },
    Bool {
        default: bool
    },
    String {
        default: String
    },
}


