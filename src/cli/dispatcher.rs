use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

use crate::{command::SubdeviceCommand, map::IglooStack, selector::{Selection, SelectorError}, VERSION};

use super::model::{Cli, CliCommands, ListItems, LogType};

#[derive(Error, Debug, Serialize)]
pub enum DispatchError {
    #[error("selector error `{0}`")]
    SelectorError(SelectorError),
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
}

impl From<SelectorError> for DispatchError {
    fn from(value: SelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl Cli {
    pub async fn dispatch(self, stack: Arc<IglooStack>) -> Result<impl IntoResponse, DispatchError> {
        Ok(match self.command {
            CliCommands::Light(args) => {
                let mut selector = Selection::new(&stack.cmd_chan_map, &args.target)?;
                selector.execute(SubdeviceCommand::Light(args.action))?;
                (StatusCode::OK).into_response()
            },
            CliCommands::Switch(_) => todo!(),
            CliCommands::UI => {
                (StatusCode::OK, Json(stack.ui_elements.clone())).into_response()
            }
            CliCommands::List(args) => {
                match args.item {
                    ListItems::Users => todo!(),
                    ListItems::UserGroups => todo!(),
                    ListItems::Providers => todo!(),
                    ListItems::Automations => todo!(),
                    ListItems::Zones => {
                        let zones: Vec<String> = stack.cmd_chan_map.keys().cloned().collect();
                        (StatusCode::OK, Json(zones)).into_response()
                    },
                    ListItems::Devices { zone } => {
                        let mut dev_names = Vec::new();
                        let zone = stack.cmd_chan_map.get(&zone).ok_or(DispatchError::UnknownZone(zone))?;
                        for (dev_name, _) in zone {
                            dev_names.push(dev_name.to_string());
                        }
                        (StatusCode::OK, Json(dev_names)).into_response()
                    },
                    ListItems::Subdevices { dev: _ } => {
                        todo!()
                    },
                }
            },
            CliCommands::Logs(args) => {
                match args.log_type {
                    LogType::System => todo!(),
                    LogType::User { user: _ } => todo!(),
                    LogType::Device { dev: _ } => {
                        todo!()
                    },
                    LogType::Automation { automation: _ } => todo!(),
                }
            },
            CliCommands::Automation(_) => todo!(),
            CliCommands::Reload => todo!(),
            CliCommands::Version => (StatusCode::OK, Json(VERSION)).into_response()
        })
    }
}

