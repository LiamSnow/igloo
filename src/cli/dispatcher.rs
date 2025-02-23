use std::{collections::HashMap, sync::Arc};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

use crate::{device::{command::{SubdeviceCommand, ScopedSubdeviceCommand}, device::{DeviceInfo, SubdeviceInfo}}, map::{IglooStack, Selector, SelectorError, UIElement}, CONFIG_VERSION, VERSION};

use super::model::{Cli, Commands, DescribeItems, ListItems, LogType};

#[derive(Error, Debug, Serialize)]
pub enum DispatchError {
    #[error("selector error `{0}`")]
    SelectorError(SelectorError),
}

impl From<SelectorError> for DispatchError {
    fn from(value: SelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl Cli {
    pub async fn dispatch(self, table: Arc<IglooStack>) -> Result<impl IntoResponse, DispatchError> {
        Ok(match self.command {
            Commands::Light(args) => {
                let cmd = ScopedSubdeviceCommand::from_str(
                    table.map.clone(),
                    &args.target,
                    SubdeviceCommand::Light(args.action)
                )?;
                cmd.execute().await;
                (StatusCode::OK).into_response()
            },
            Commands::Switch(_) => todo!(),
            Commands::UI => {
                (StatusCode::OK, Json(table.ui)).into_response()
            }
            Commands::List(args) => {
                match args.item {
                    ListItems::Users => todo!(),
                    ListItems::UserGroups => todo!(),
                    ListItems::Providers => todo!(),
                    ListItems::Automations => todo!(),
                    ListItems::Zones => {
                        let zones: Vec<String> = table.map.keys().cloned().collect();
                        (StatusCode::OK, Json(zones)).into_response()
                    },
                    ListItems::Devices { zone } => {
                        let zone = Selector::from_str(table.map.clone(), &zone)?.get_zone()?;
                        let devices: Vec<String> = zone.keys().cloned().collect();
                        (StatusCode::OK, Json(devices)).into_response()
                    },
                    ListItems::Subdevices { dev } => {
                        let dev_lock = Selector::from_str(table.map.clone(), &dev)?.get_device()?;
                        let dev = dev_lock.read().await;
                        (StatusCode::OK, Json(dev.list_subdevs())).into_response()
                    },
                }
            },
            Commands::Describe(args) => {
                match args.item {
                    DescribeItems::Device { dev } => {
                        let dev_lock = Selector::from_str(table.map.clone(), &dev)?.get_device()?;
                        let dev = dev_lock.read().await;
                        (StatusCode::OK, Json(dev.describe())).into_response()
                    },
                    DescribeItems::Automation { automation: _ } => {
                        todo!()
                    },
                }
            },
            Commands::Logs(args) => {
                match args.log_type {
                    LogType::System => todo!(),
                    LogType::User { user: _ } => todo!(),
                    LogType::Device { dev } => {
                        let dev_lock = Selector::from_str(table.map.clone(), &dev)?.get_device()?;
                        let mut dev = dev_lock.write().await;
                        dev.subscribe_logs().await;
                        todo!()
                    },
                    LogType::Automation { automation: _ } => todo!(),
                }
            },
            Commands::Automation(_) => todo!(),
            Commands::Reload => todo!(),
            Commands::Version => (StatusCode::OK, Json(VERSION)).into_response()
        })
    }
}

