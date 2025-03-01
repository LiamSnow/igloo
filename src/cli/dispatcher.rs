use std::{collections::HashMap, sync::Arc};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

use crate::{command::{AveragedSubdeviceState, SubdeviceCommand}, config::{parse_time, ElementValue}, effects, map::{Element, IglooStack}, selector::{DeviceChannelError, Selection, SelectorError}, VERSION};

use super::model::{Cli, CliCommands, ListItems, LogType, UICommand};

#[derive(Error, Debug, Serialize)]
pub enum DispatchError {
    #[error("selector error `{0}`")]
    SelectorError(SelectorError),
    #[error("invalid element value selector `{0}`")]
    InvalidElementValueSelector(String),
    #[error("selector `{0}` had channel error `{1}`")]
    DeviceChannelErorr(String, DeviceChannelError),
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
}

impl From<SelectorError> for DispatchError {
    fn from(value: SelectorError) -> Self {
        Self::SelectorError(value)
    }
}

#[derive(Serialize)]
struct UIResponse<'a> {
    elements: &'a HashMap<String, Vec<Element>>,
    states: &'a Vec<Option<AveragedSubdeviceState>>,
    values: &'a Vec<ElementValue>
}

impl Cli {
    pub async fn dispatch(self, stack: Arc<IglooStack>) -> Result<impl IntoResponse, DispatchError> {
        Ok(match self.command {
            CliCommands::Light(args) => {
                let selection = Selection::from_str(&stack.lut, &args.target)?;
                effects::clear_conflicting(&stack, &selection, &((&args.action).into())).await;
                selection.execute(&stack, SubdeviceCommand::Light(args.action))
                    .map_err(|e| DispatchError::DeviceChannelErorr(args.target, e))?;
                (StatusCode::OK).into_response()
            },
            CliCommands::Effect(args) => {
                let selection = Selection::from_str(&stack.lut, &args.target)?;
                effects::spawn(stack.clone(), selection, args.effect).await;
                (StatusCode::OK).into_response()
            },
            CliCommands::Switch(_) => todo!(),
            CliCommands::UI(arg) => match arg.arg {
                UICommand::Get => {
                    let states = stack.element_states.lock().await;
                    let values = stack.element_values.lock().await;
                    (StatusCode::OK, Json(UIResponse {
                        elements: &stack.elements,
                        states: &states,
                        values: &values,
                    })).into_response()
                },
                UICommand::Set { selector, value } => {
                    let evid = stack.evid_lut.get(&selector)
                        .ok_or(DispatchError::InvalidElementValueSelector(selector))?;
                    let mut evs = stack.element_values.lock().await;
                    match evs.get_mut(*evid).unwrap() {
                        ElementValue::Time(naive_time) => *naive_time = parse_time(&value).unwrap(),
                    }
                    //TODO notif automations
                    (StatusCode::OK).into_response()
                },
            }
            CliCommands::List(args) => {
                match args.item {
                    ListItems::Users => todo!(),
                    ListItems::UserGroups => todo!(),
                    ListItems::Providers => todo!(),
                    ListItems::Automations => todo!(),
                    ListItems::Zones => {
                        let zones: Vec<_> = stack.lut.zid.keys().collect();
                        (StatusCode::OK, Json(zones)).into_response()
                    },
                    ListItems::Devices { zone } => {
                        let zid = stack.lut.zid.get(&zone).ok_or(DispatchError::UnknownZone(zone))?;
                        let names: Vec<_> = stack.lut.did.get(*zid).unwrap().keys().collect();
                        (StatusCode::OK, Json(names)).into_response()
                    },
                    ListItems::Subdevices { dev: _ } => {
                        todo!()
                    },
                    ListItems::Effects { target } => {
                        let selection = match target {
                            Some(target) => Selection::from_str(&stack.lut, &target)?,
                            None => Selection::All,
                        };
                        let res = effects::list(&stack, &selection).await;
                        (StatusCode::OK, Json(res)).into_response()
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

