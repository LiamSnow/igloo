use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    command::SubdeviceCommand, elements::{parse_time, AveragedSubdeviceState, Element, ElementValue, ElementValueUpdate}, stack::IglooStack, scripts::{self, error::ScriptError, ScriptCancelFailure}, selector::{DeviceChannelError, Selection, SelectorError}, VERSION
};

use super::model::{Cli, CliCommands, ListItems, LogType, ScriptAction, UICommand};

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
    #[error("you do not have permission to perform this operation")]
    InvalidPermission,
    #[error("script `${0}` currently has ownership and cannot be cancelled")]
    UncancellableScript(String),
    #[error("script error `${0}`")]
    ScriptError(ScriptError),
    #[error("script cancel error `${0}`")]
    ScriptCancelFailure(ScriptCancelFailure),
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

impl From<ScriptError> for DispatchError {
    fn from(value: ScriptError) -> Self {
        Self::ScriptError(value)
    }
}

#[derive(Serialize)]
struct UIResponse<'a> {
    elements: Vec<(&'a String, Vec<&'a Element>)>,
    states: &'a Vec<Option<AveragedSubdeviceState>>,
    values: &'a Vec<ElementValue>,
}

impl Cli {
    pub async fn dispatch(
        self,
        stack: &Arc<IglooStack>,
        uid: usize,
        cancel_conflicting: bool,
    ) -> Result<Option<String>, DispatchError> {
        // prechecks
        let sel = match self.command.get_selection() {
            Some(sel_str) => {
                let sel = Selection::from_str(&stack.dev_lut, &sel_str)?;

                //check permissions
                if !stack.perms.has_perm(&sel, uid) {
                    return Err(DispatchError::InvalidPermission);
                }

                //try to cancel conflicting scripts
                if cancel_conflicting {
                    if let Some(subdev_type) = self.command.get_subdev_type() {
                        let res = scripts::clear_conflicting_for_cmd(&stack, &sel, &subdev_type).await;
                        if let Some(scr) = res {
                            return Err(DispatchError::UncancellableScript(scr));
                        }
                    }
                }

                Some(sel)
            }
            None => None,
        };

        // dispatch
        Ok(match self.command {
            CliCommands::Light(args) => {
                sel.unwrap()
                    .execute(&stack, SubdeviceCommand::Light(args.action))
                    .map_err(|e| DispatchError::DeviceChannelErorr(args.target, e))?;
                None
            }
            CliCommands::Switch(args) => {
                sel.unwrap()
                    .execute(&stack, SubdeviceCommand::Switch(args.action))
                    .map_err(|e| DispatchError::DeviceChannelErorr(args.target, e))?;
                None
            }
            CliCommands::Script(args) => {
                match args.action {
                    ScriptAction::Run { extra_args, name } => {
                        scripts::spawn(&stack.clone(), name, extra_args, uid).await?;
                    }
                    ScriptAction::CancelAll { name } => {
                        if let Some(failure) = scripts::cancel_all(stack, &name, uid).await {
                            return Err(DispatchError::ScriptCancelFailure(failure))
                        }
                    }
                    ScriptAction::Cancel { id } => {
                        if let Some(failure) = scripts::cancel(stack, id, uid).await {
                            return Err(DispatchError::ScriptCancelFailure(failure))
                        }
                    },
                }
                None
            }
            CliCommands::UI(arg) => match arg.arg {
                UICommand::Get => {
                    //remove not allowed elements
                    let mut elements = Vec::new();
                    for (group_name, els) in &stack.elements.elements {
                        let mut els_for_user = Vec::new();
                        for el in els {
                            let allowed = match &el.allowed_uids {
                                Some(uids) => *uids.get(uid).unwrap(),
                                None => true,
                            };
                            if allowed {
                                els_for_user.push(el);
                            }
                        }
                        if els_for_user.len() > 0 {
                            elements.push((group_name, els_for_user));
                        }
                    }

                    let states = stack.elements.states.lock().await;
                    let values = stack.elements.values.lock().await;
                    let res = UIResponse {
                        elements,
                        states: &states,
                        values: &values,
                    };
                    Some(serde_json::to_string(&res)?)
                }
                UICommand::Set { selector, value } => {
                    let evid = *stack
                        .elements
                        .evid_lut
                        .get(&selector)
                        .ok_or(DispatchError::InvalidElementValueSelector(selector.clone()))?;
                    let mut evs = stack.elements.values.lock().await;
                    match evs.get_mut(evid).unwrap() {
                        //FIXME
                        ElementValue::Time(naive_time) => *naive_time = parse_time(&value).unwrap(), //FIXME
                    }
                    let u = ElementValueUpdate {
                        evid,
                        value: evs.get(evid).unwrap().clone(),
                    };
                    stack
                        .ws_broadcast
                        .send(serde_json::to_string(&u).unwrap().into())
                        .unwrap(); //FIXME
                                   //TODO notif automations
                    None
                }
            },
            CliCommands::List(args) => match args.item {
                ListItems::Users => todo!(),
                ListItems::UserGroups => todo!(),
                ListItems::Providers => todo!(),
                ListItems::Zones => {
                    let zones: Vec<_> = stack.dev_lut.zid.keys().collect();
                    Some(serde_json::to_string(&zones)?)
                }
                ListItems::Devices { zone } => {
                    let zid = stack
                        .dev_lut
                        .zid
                        .get(&zone)
                        .ok_or(DispatchError::UnknownZone(zone))?;
                    let names: Vec<_> = stack.dev_lut.did.get(*zid).unwrap().keys().collect();
                    Some(serde_json::to_string(&names)?)
                }
                ListItems::Subdevices { dev: _ } => {
                    todo!()
                }
                ListItems::Scripts => {
                    let res = stack.script_states.lock().await;
                    Some(serde_json::to_string(&res.current)?)
                },
            },
            CliCommands::Logs(args) => match args.log_type {
                LogType::System => todo!(),
                LogType::Device { name: _ } => {
                    todo!()
                }
                LogType::Script { name: _ } => todo!(),
            },
            CliCommands::Reload => todo!(),
            CliCommands::Version => Some(serde_json::to_string(&VERSION)?),
        })
    }
}
