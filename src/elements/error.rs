use serde::Serialize;
use thiserror::Error;

use crate::{device::error::DeviceSelectorError, scripts::error::ScriptError};

#[derive(Error, Debug, Serialize)]
pub enum ElementsInitError {
    #[error("bad selector `{0}`")]
    BadSelector(DeviceSelectorError),
    #[error("invalid button command `{0}`")]
    InvalidButtonCommand(String),
    #[error("script error `{0}`")]
    ScriptError(ScriptError),
}

impl From<DeviceSelectorError> for ElementsInitError {
    fn from(value: DeviceSelectorError) -> Self {
        Self::BadSelector(value)
    }
}

impl From<ScriptError> for ElementsInitError {
    fn from(value: ScriptError) -> Self {
        Self::ScriptError(value)
    }
}
