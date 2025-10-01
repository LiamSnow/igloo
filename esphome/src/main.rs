use crate::floe::ESPHomeFloe;
use igloo_interface::floe::{FloeManager, FloeManagerError};
use tokio::fs;

pub mod connection;
pub mod device;
pub mod floe;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}
pub mod model {
    include!(concat!(env!("OUT_DIR"), "/model.rs"));
}

pub const CONFIG_FILE: &str = "./data/config.json";

#[tokio::main]
async fn main() -> Result<(), FloeManagerError> {
    let contents = fs::read_to_string(CONFIG_FILE).await.unwrap(); // FIXME unwrap
    let handler = ESPHomeFloe {
        config: serde_json::from_str(&contents).unwrap(), // FIXME unwrap
        ..Default::default()
    };
    FloeManager::run(handler).await?;
    Ok(())
}
