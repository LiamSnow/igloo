use ron::ser::PrettyConfig;
use thiserror::Error;
use tokio::{fs, io};

use super::model::Auth;

const FILE: &str = "auth.ron";

#[derive(Error, Debug)]
pub enum AuthFileError {
    #[error("file system error: {0}")]
    FileSystem(#[from] io::Error),
    #[error("ron deserialize error: {0}")]
    RonDeserialize(#[from] ron::de::SpannedError),
    #[error("ron serialize error: {0}")]
    Ron(#[from] ron::error::Error),
}

impl Auth {
    pub async fn load() -> Result<Self, AuthFileError> {
        if fs::try_exists(FILE).await? {
            let contents = fs::read_to_string(FILE).await?;
            let res = ron::from_str(&contents)?;
            Ok(res)
        } else {
            // TODO change to make blank
            println!("{FILE} doesn't exist, making test data.");

            let mut auth = Auth::new();

            auth.add_user("liamsnow".to_string(), "test123".to_string())
                .unwrap();

            fs::write(FILE, ron::to_string(&auth)?).await?;

            Ok(auth)
        }
    }

    pub async fn save(&self) -> Result<(), AuthFileError> {
        let contents = ron::ser::to_string_pretty(self, PrettyConfig::new())?;
        fs::write(FILE, contents).await?;
        Ok(())
    }
}
