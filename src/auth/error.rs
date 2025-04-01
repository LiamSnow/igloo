use thiserror::Error;

use crate::selector::SelectorError;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("selector error `{0}`")]
    SelectorError(SelectorError),
    #[error("sqlite error `{0}`")]
    SQLiteError(tokio_rusqlite::Error),
}

impl From<SelectorError> for AuthError {
    fn from(value: SelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl From<tokio_rusqlite::Error> for AuthError {
    fn from(value: tokio_rusqlite::Error) -> Self {
        Self::SQLiteError(value)
    }
}
