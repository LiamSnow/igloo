use igloo_interface::Device;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tokio::{fs, io};
use uuid::Uuid;

const FILE: &str = "state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GlacierState {
    pub zones: HashMap<Uuid, Zone>,
    #[serde(skip)]
    pub devices: HashMap<Uuid, Device>,
    /// maps Device ID -> Provider Name
    #[serde(skip)]
    pub providers: HashMap<Uuid, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub devices: Vec<Uuid>,
}

#[derive(Error, Debug)]
pub enum StateFileError {
    #[error("file system error: {0}")]
    FileSystem(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl GlacierState {
    pub async fn load() -> Result<Self, StateFileError> {
        if fs::try_exists(FILE).await? {
            let contents = fs::read_to_string(FILE).await?;
            let res = serde_json::from_str(&contents)?;
            Ok(res)
        } else {
            // TODO change to make blank
            println!("{FILE} doesn't exist, making test data.");

            let mut state = GlacierState::default();

            state.zones.insert(
                Uuid::now_v7(),
                Zone {
                    name: "Kitchen".to_string(),
                    devices: vec![],
                },
            );

            state.save().await?;

            Ok(state)
        }
    }

    pub async fn save(&self) -> Result<(), StateFileError> {
        let contents = serde_json::to_string(self)?;
        fs::write(FILE, contents).await?;
        Ok(())
    }
}
