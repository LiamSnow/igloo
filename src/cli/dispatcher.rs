use std::sync::Arc;

use serde::Serialize;

use crate::{
    elements::Element, entity::{self, AveragedEntityState}, scripts, selector::Selection,
    state::IglooState, VERSION,
};

use super::{
    error::DispatchError,
    model::{Cli, CliCommands, ListItems, LogType, ScriptAction},
};

#[derive(Serialize)]
struct UIResponse<'a> {
    elements: Vec<(&'a String, Vec<&'a Element>)>,
    states: &'a Vec<Option<AveragedEntityState>>,
}

impl Cli {
    pub async fn dispatch(
        self,
        state: &Arc<IglooState>,
        uid: usize,
        cancel_conflicting: bool,
    ) -> Result<Option<String>, DispatchError> {
        let sel = precheck_selection(&self, state, uid, cancel_conflicting).await?;
        Ok(match self.command {
            CliCommands::Light(args) => entity::light::dispatch(args.action, args.target, sel.unwrap(), state)?,
            CliCommands::Int(args) => entity::int::dispatch(args.value, args.target, sel.unwrap(), state)?,
            CliCommands::Float(args) => entity::float::dispatch(args.value, args.target, sel.unwrap(), state)?,
            CliCommands::Bool(args) => entity::bool::dispatch(args.action, args.target, sel.unwrap(), state)?,
            CliCommands::Text(args) => entity::text::dispatch(args.value, args.target, sel.unwrap(), state)?,
            CliCommands::Time(args) => entity::time::dispatch(args.value, args.target, sel.unwrap(), state)?,
            CliCommands::Script(args) => args.action.dispatch(state, uid).await?,
            CliCommands::UI => get_ui_for_user(state, uid).await?,
            CliCommands::List(args) => args.item.dispatch(state).await?,
            CliCommands::Logs(args) => args.log_type.dispatch(state).await?,
            CliCommands::Reload => todo!(),
            CliCommands::Version => Some(serde_json::to_string(&VERSION)?),
        })
    }
}

async fn precheck_selection(
    cmd: &Cli,
    state: &Arc<IglooState>,
    uid: usize,
    cancel_conflicting: bool,
) -> Result<Option<Selection>, DispatchError> {
    Ok(match cmd.command.get_selection() {
        Some(sel_str) => {
            let sel = Selection::from_str(&state.devices.lut, &sel_str)?;

            //check permissions
            if !state.auth.is_authorized(&sel, uid) {
                return Err(DispatchError::InvalidPermission);
            }

            //try to cancel conflicting scripts
            if cancel_conflicting {
                if let Some(entity_type) = cmd.command.get_entity_type() {
                    let res = scripts::clear_conflicting_for_cmd(&state, &sel, &entity_type).await;
                    if let Some(scr) = res {
                        return Err(DispatchError::UncancellableScript(scr));
                    }
                }
            }

            Some(sel)
        }
        None => None,
    })
}

impl ScriptAction {
    async fn dispatch(
        self,
        state: &Arc<IglooState>,
        uid: usize,
    ) -> Result<Option<String>, DispatchError> {
        match self {
            ScriptAction::Run { extra_args, name } => {
                scripts::spawn(&state.clone(), name, extra_args, uid).await?;
            }
            ScriptAction::CancelAll { name } => {
                if let Some(failure) = scripts::cancel_all(state, &name, uid).await {
                    return Err(DispatchError::ScriptCancelFailure(failure));
                }
            }
            ScriptAction::Cancel { id } => {
                if let Some(failure) = scripts::cancel(state, id, uid).await {
                    return Err(DispatchError::ScriptCancelFailure(failure));
                }
            }
        }
        Ok(None)
    }
}

async fn get_ui_for_user(
    state: &Arc<IglooState>,
    uid: usize,
) -> Result<Option<String>, DispatchError> {
    //remove unauthorized elements
    let mut elements = Vec::new();
    for (group_name, els) in &state.elements.elements {
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

    let states = state.elements.states.lock().await;
    let res = UIResponse {
        elements,
        states: &states,
    };
    Ok(Some(serde_json::to_string(&res)?))
}

impl ListItems {
    async fn dispatch(self, state: &Arc<IglooState>) -> Result<Option<String>, DispatchError> {
        Ok(match self {
            ListItems::Users => todo!(),
            ListItems::UserGroups => todo!(),
            ListItems::Providers => todo!(),
            ListItems::Zones => {
                let zones: Vec<_> = state.devices.lut.zid.keys().collect();
                Some(serde_json::to_string(&zones)?)
            }
            ListItems::Devices { zone } => {
                let zid = state
                    .devices
                    .lut
                    .zid
                    .get(&zone)
                    .ok_or(DispatchError::UnknownZone(zone))?;
                let names: Vec<_> = state.devices.lut.did.get(*zid).unwrap().keys().collect();
                Some(serde_json::to_string(&names)?)
            }
            ListItems::Entities { dev: _ } => {
                todo!()
            }
            ListItems::Scripts => {
                let res = state.scripts.states.lock().await;
                Some(serde_json::to_string(&res.current)?)
            }
        })
    }
}

impl LogType {
    async fn dispatch(self, _state: &Arc<IglooState>) -> Result<Option<String>, DispatchError> {
        match self {
            LogType::System => todo!(),
            LogType::Device { name: _ } => {
                todo!()
            }
            LogType::Script { name: _ } => todo!(),
        }
    }
}
