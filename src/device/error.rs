use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;

use crate::entity::TargetedEntityCommand;

#[derive(Error, Debug, Serialize)]
pub enum DeviceSelectorError {
    #[error("scope selector must be `all`, ZONE, ZONE.DEVICE, or ZONE.DEVICE.SUBDEVICE")]
    BadSelector,
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
    #[error("unknown device `{0}.{1}`")]
    UnknownDevice(String, String),
}

#[derive(Error, Debug, Serialize)]
pub enum DeviceChannelError {
    #[error("full")]
    Full,
    #[error("closed")]
    Closed,
}

impl From<TrySendError<TargetedEntityCommand>> for DeviceChannelError {
    fn from(value: TrySendError<TargetedEntityCommand>) -> Self {
        match value {
            TrySendError::Full(_) => Self::Full,
            TrySendError::Closed(_) => Self::Closed,
        }
    }
}
