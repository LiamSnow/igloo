use thiserror::Error;

use crate::device::error::DeviceSelectorError;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("selector error `{0}`")]
    SelectorError(DeviceSelectorError),
    #[error("sqlite error `{0}`")]
    SQLiteError(tokio_rusqlite::Error),
}

impl From<DeviceSelectorError> for AuthError {
    fn from(value: DeviceSelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl From<tokio_rusqlite::Error> for AuthError {
    fn from(value: tokio_rusqlite::Error) -> Self {
        Self::SQLiteError(value)
    }
}
