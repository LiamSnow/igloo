use thiserror::Error;
use tokio::{fs, io};

use super::model::Auth;

const FILE: &str = "auth.json";

#[derive(Error, Debug)]
pub enum AuthFileError {
    #[error("file system error: {0}")]
    FileSystem(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Auth {
    pub async fn load() -> Result<Self, AuthFileError> {
        if fs::try_exists(FILE).await? {
            let contents = fs::read_to_string(FILE).await?;
            let res = serde_json::from_str(&contents)?;
            Ok(res)
        } else {
            // TODO change to make blank
            println!("{FILE} doesn't exist, making test data.");

            let mut auth = Auth::new();

            auth.add_user("liamsnow".to_string(), "test123".to_string())
                .unwrap();

            auth.save().await?;

            Ok(auth)
        }
    }

    pub async fn save(&self) -> Result<(), AuthFileError> {
        let contents = serde_json::to_string(self)?;
        fs::write(FILE, contents).await?;
        Ok(())
    }
}
