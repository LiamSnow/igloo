use std::collections::HashMap;

use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceConfig {
    username: String,
    password: String,
    port: u16,
    /// subdev_name -> mqtt path
    /// creates `text` subdevices
    publish: HashMap<String, String>,
    /// mqtt path -> cmd
    subscribe: HashMap<String, String>,
}
