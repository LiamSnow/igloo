use serde::Serialize;
use thiserror::Error;

use crate::selector::SelectorError;


#[derive(Error, Debug, Serialize)]
pub enum ScriptError {
    #[error("selector error `{0}`")]
    SelectorError(SelectorError),
    #[error("expected at least `${1}` args but got `${0}`")]
    NotEnoughArgs(usize, usize),
    #[error("unknown script `{0}`")]
    UnknownScript(String),
    #[error("bad positional arg specifier `{0}`")]
    BadPositionalArgSpecifier(String),
    #[error("claim is empty")]
    ClaimIsEmpty,
    #[error("builtin failure")]
    BuiltInFailure(String),
    #[error("could not cancel other script `${0}`")]
    CouldNotCancel(String),
    #[error("not authorized")]
    NotAuthorized,
}

impl From<SelectorError> for ScriptError {
    fn from(value: SelectorError) -> Self {
        ScriptError::SelectorError(value)
    }
}

