use serde::Serialize;
use thiserror::Error;

use crate::{device::error::{DeviceChannelError, DeviceSelectorError}, scripts::{error::ScriptError, ScriptCancelFailure}};

#[derive(Error, Debug, Serialize)]
pub enum DispatchError {
    #[error("selector error `{0}`")]
    SelectorError(DeviceSelectorError),
    #[error("invalid element value selector `{0}`")]
    InvalidElementValueSelector(String),
    #[error("selector `{0}` had channel error `{1}`")]
    DeviceChannelError(String, DeviceChannelError),
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
    #[error("json encoding error `{0}`")]
    JsonEncodingError(String),
    #[error("you do not have permission to perform this operation")]
    InvalidPermission,
    #[error("script `${0}` currently has ownership and cannot be cancelled")]
    UncancellableScript(String),
    #[error("script error `${0}`")]
    ScriptError(ScriptError),
    #[error("script cancel error `${0}`")]
    ScriptCancelFailure(ScriptCancelFailure),
    #[error("command requires UID")]
    CommandRequiresUID,
}

impl From<serde_json::Error> for DispatchError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonEncodingError(value.to_string())
    }
}

impl From<DeviceSelectorError> for DispatchError {
    fn from(value: DeviceSelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl From<ScriptError> for DispatchError {
    fn from(value: ScriptError) -> Self {
        Self::ScriptError(value)
    }
}

