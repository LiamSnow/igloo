use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use thiserror::Error;

use crate::{
    command::{AveragedSubdeviceState, SubdeviceCommand},
    config::{parse_time, ElementValue},
    effects,
    map::{Element, IglooStack},
    selector::{DeviceChannelError, Selection, SelectorError},
    VERSION,
};

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
    #[error("json encoding error `{0}`")]
    JsonEncodingError(String),
}

impl From<serde_json::Error> for DispatchError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonEncodingError(value.to_string())
    }
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
    values: &'a Vec<ElementValue>,
}

impl Cli {
    pub async fn dispatch(
        self,
        stack: &Arc<IglooStack>,
    ) -> Result<Option<String>, DispatchError> {
        Ok(match self.command {
            CliCommands::Light(args) => {
                let selection = Selection::from_str(&stack.lut, &args.target)?;
                effects::clear_conflicting(&stack, &selection, &((&args.action).into())).await;
                selection
                    .execute(&stack, SubdeviceCommand::Light(args.action))
                    .map_err(|e| DispatchError::DeviceChannelErorr(args.target, e))?;
                None
            }
            CliCommands::Effect(args) => {
                let selection = Selection::from_str(&stack.lut, &args.target)?;
                effects::spawn(stack.clone(), selection, args.effect).await;
                None
            }
            CliCommands::Switch(_) => todo!(),
            CliCommands::UI(arg) => match arg.arg {
                UICommand::Get => {
                    let states = stack.element_states.lock().await;
                    let values = stack.element_values.lock().await;
                    let res = UIResponse {
                        elements: &stack.elements,
                        states: &states,
                        values: &values,
                    };
                    Some(serde_json::to_string(&res)?)
                }
                UICommand::Set { selector, value } => {
                    let evid = stack
                        .evid_lut
                        .get(&selector)
                        .ok_or(DispatchError::InvalidElementValueSelector(selector))?;
                    let mut evs = stack.element_values.lock().await;
                    match evs.get_mut(*evid).unwrap() {
                        ElementValue::Time(naive_time) => *naive_time = parse_time(&value).unwrap(),
                    }
                    //TODO notif automations
                    None
                }
            },
            CliCommands::List(args) => match args.item {
                ListItems::Users => todo!(),
                ListItems::UserGroups => todo!(),
                ListItems::Providers => todo!(),
                ListItems::Automations => todo!(),
                ListItems::Zones => {
                    let zones: Vec<_> = stack.lut.zid.keys().collect();
                    Some(serde_json::to_string(&zones)?)
                }
                ListItems::Devices { zone } => {
                    let zid = stack
                        .lut
                        .zid
                        .get(&zone)
                        .ok_or(DispatchError::UnknownZone(zone))?;
                    let names: Vec<_> = stack.lut.did.get(*zid).unwrap().keys().collect();
                    Some(serde_json::to_string(&names)?)
                }
                ListItems::Subdevices { dev: _ } => {
                    todo!()
                }
                ListItems::Effects { target } => {
                    let selection = match target {
                        Some(target) => Selection::from_str(&stack.lut, &target)?,
                        None => Selection::All,
                    };
                    let res = effects::list(&stack, &selection).await;
                    Some(serde_json::to_string(&res)?)
                }
            },
            CliCommands::Logs(args) => match args.log_type {
                LogType::System => todo!(),
                LogType::User { user: _ } => todo!(),
                LogType::Device { dev: _ } => {
                    todo!()
                }
                LogType::Automation { automation: _ } => todo!(),
            },
            CliCommands::Automation(_) => todo!(),
            CliCommands::Reload => todo!(),
            CliCommands::Version => Some(serde_json::to_string(&VERSION)?)
        })
    }
}
