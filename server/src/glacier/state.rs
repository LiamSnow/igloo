use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tokio::{fs, io};
use uuid::Uuid;

const FILE: &str = "state.ron";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub zones: HashMap<Uuid, Zone>,
    pub devices: HashMap<Uuid, Device>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Zone {
    pub name: String,
    pub devices: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub provider: Uuid,
    pub components: HashMap<String, Component>,
    // last_seen, enabled, ...?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub r#type: igloo_interface::Type,
    pub r#value: igloo_interface::Value,
}

#[derive(Error, Debug)]
pub enum StateFileError {
    #[error("file system error: {0}")]
    FileSystem(#[from] io::Error),
    #[error("ron deserialize error: {0}")]
    RonDeserialize(#[from] ron::de::SpannedError),
    #[error("ron serialize error: {0}")]
    Ron(#[from] ron::error::Error),
}

impl State {
    pub async fn load() -> Result<Self, StateFileError> {
        if fs::try_exists(FILE).await? {
            let contents = fs::read_to_string(FILE).await?;
            let res = ron::from_str(&contents)?;
            Ok(res)
        } else {
            // TODO change to make blank
            println!("{FILE} doesn't exist, making test data.");

            let mut state = State::default();

            fs::write(FILE, ron::to_string(&state)?).await?;

            Ok(state)
        }
    }

    pub async fn save(&self) -> Result<(), StateFileError> {
        let contents = ron::ser::to_string_pretty(self, PrettyConfig::new())?;
        fs::write(FILE, contents).await?;
        Ok(())
    }
}
