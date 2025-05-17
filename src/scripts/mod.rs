use std::{collections::HashMap, sync::Arc};

use bitvec::prelude::bitvec;
use bitvec::vec::BitVec;
use error::ScriptError;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::{oneshot, Mutex};
use tracing::{info, span, Level};

use crate::{
    auth::Auth, config::ScriptConfig, device::ids::{DeviceIDLut, DeviceIDSelection}, elements, entity::EntityType,
    state::IglooState,
};

mod basic;
mod builtin;
pub mod error;
mod python;

pub struct Scripts {
    pub states: Mutex<ScriptStates>,
    pub configs: HashMap<String, ScriptConfig>,
}

impl Scripts {
    pub fn init(configs: HashMap<String, ScriptConfig>) -> Self {
        Self {
            states: Mutex::new(ScriptStates::default()),
            configs,
        }
    }
}

#[derive(Default)]
pub struct ScriptStates {
    pub next_id: u32,
    pub current: HashMap<u32, RunningScriptMeta>,
}

pub type ScriptClaims = HashMap<EntityType, Vec<String>>;

pub struct ScriptMeta<'a> {
    pub claims: &'a ScriptClaims,
    pub auto_cancel: bool,
    pub auto_run: bool,
}

#[derive(Serialize)]
pub struct RunningScriptMeta {
    script_name: String,
    #[serde(skip_serializing)]
    claims: HashMap<EntityType, Vec<DeviceIDSelection>>,
    auto_cancel: bool,
    #[serde(skip_serializing)]
    cancel_tx: Option<oneshot::Sender<()>>,
    #[serde(skip_serializing)]
    perms: BitVec,
}

pub async fn spawn_boot(state: &Arc<IglooState>) -> Result<(), ScriptError> {
    for (name, cfg) in &state.scripts.configs {
        if cfg.get_meta().auto_run {
            spawn(&state.clone(), name.to_string(), vec![], None, None).await?;
        }
    }

    for name in builtin::get_boot() {
        spawn(&state.clone(), name.to_string(), vec![], None, None).await?;
    }

    Ok(())
}

pub async fn spawn(
    state: &Arc<IglooState>,
    script_name: String,
    args: Vec<String>,
    id: Option<u32>,
    uid: Option<usize>,
) -> Result<(), ScriptError> {
    if let Some(meta) = builtin::get_meta(&script_name) {
        let span = span!(Level::INFO, "Builtin Script", script_name, id);
        let _enter = span.enter();
        info!("running uid={:#?}, args={:#?}", uid, args);

        let (id, cancel_rx) = init_script(state, uid, &script_name, &args, id, meta).await?;

        let res = builtin::spawn(&script_name, id, state.clone(), uid, args, cancel_rx).await;
        if let Err(err) = res {
            tracing::error!("{}", err.to_string());
            return Err(ScriptError::BuiltInFailure(err.to_string()));
        }

        Ok(())
    } else if let Some(cfg) = state.scripts.configs.get(&script_name) {
        let meta = cfg.get_meta();
        let (id, cancel_rx) = init_script(state, uid, &script_name, &args, id, meta).await?;

        match cfg {
            ScriptConfig::Python(cfg) => python::spawn(
                script_name,
                id,
                state.clone(),
                uid,
                args,
                cancel_rx,
                cfg.file.clone(),
            ),
            ScriptConfig::Basic(cfg) => basic::spawn(
                script_name,
                id,
                state.clone(),
                uid,
                args,
                cancel_rx,
                cfg.body.clone(),
            ),
        }

        Ok(())
    } else {
        Err(ScriptError::UnknownScript(script_name))
    }
}

pub fn calc_perms(
    dev_lut: &DeviceIDLut,
    auth: &Auth,
    script_configs: &HashMap<String, ScriptConfig>,
    script_name: String,
    args: Vec<String>,
) -> Result<BitVec, ScriptError> {
    let meta = match builtin::get_meta(&script_name) {
        Some(meta) => meta,
        None => match script_configs.get(&script_name) {
            Some(cfg) => cfg.get_meta(),
            None => return Err(ScriptError::UnknownScript(script_name)),
        },
    };

    let claims = parse_claims(&dev_lut, &meta.claims, &args)?;
    Ok(claims_to_perms(&auth, &claims))
}

async fn init_script<'a>(
    state: &Arc<IglooState>,
    uid: Option<usize>,
    script_name: &str,
    args: &Vec<String>,
    id: Option<u32>,
    meta: ScriptMeta<'a>,
) -> Result<(u32, oneshot::Receiver<()>), ScriptError> {
    let claims = parse_claims(&state.devices.lut, &meta.claims, &args)?;

    let perms = claims_to_perms(&state.auth, &claims);
    if uid.is_some() && !*perms.get(uid.unwrap()).unwrap() {
        return Err(ScriptError::NotAuthorized);
    }

    let res = clear_conflicting_for_script(state, &claims).await;
    if let Some(scr) = res {
        return Err(ScriptError::CouldNotCancel(scr));
    }

    let (cancel_tx, cancel_rx) = oneshot::channel();

    //push to state
    let mut states = state.scripts.states.lock().await;

    let id = match id {
        Some(id) => id,
        None => {
            let id = states.next_id;
            states.next_id += 1;
            id
        }
    };

    states.current.insert(
        id,
        RunningScriptMeta {
            script_name: script_name.to_string(),
            claims,
            auto_cancel: meta.auto_cancel,
            cancel_tx: Some(cancel_tx),
            perms,
        },
    );

    send_change_to_ui(state, ScriptStateChange::Add(id));

    Ok((id, cancel_rx))
}

#[derive(Serialize)]
pub enum ScriptStateChange {
    Add(u32),
    Remove(u32),
}

pub fn send_change_to_ui(state: &Arc<IglooState>, change: ScriptStateChange) {
    elements::state::broadcast_changes_to_ws(&state.elements, "scripts", &change);
}

/// Parses String claims into Selections.
/// Replaces positional args (IE $1) with their values
fn parse_claims(
    devids: &DeviceIDLut,
    raw: &HashMap<EntityType, Vec<String>>,
    extra_args: &Vec<String>,
) -> Result<HashMap<EntityType, Vec<DeviceIDSelection>>, ScriptError> {
    let mut res = HashMap::new();
    for (entity_type, sel_strs) in raw {
        let mut v = Vec::new();
        for sel_str in sel_strs {
            let sel_str = match sel_str.starts_with('$') {
                true => {
                    let idx: usize = (&sel_str[1..])
                        .parse()
                        .map_err(|_| ScriptError::BadPositionalArgSpecifier(sel_str.to_string()))?;
                    extra_args
                        .get(idx - 1)
                        .ok_or(ScriptError::NotEnoughArgs(extra_args.len(), idx))?
                }
                false => &sel_str,
            };

            v.push(DeviceIDSelection::from_str(devids, sel_str)?);
        }
        res.insert(entity_type.clone(), v);
    }
    Ok(res)
}

/// Tries to stop scripts that currently claim ownership over the same device(s)
/// Returns Some(scripe_name) if that script is conflicting and cannot be cancelled
pub async fn clear_conflicting_for_cmd(
    state: &Arc<IglooState>,
    selection: &DeviceIDSelection,
    for_type: &EntityType,
) -> Option<String> {
    let mut state = state.scripts.states.lock().await;
    for (_, meta) in &mut state.current {
        if let Some(claim_sels) = meta.claims.get(for_type) {
            if selection.collides_with_any(claim_sels) {
                if meta.auto_cancel {
                    let _ = meta.cancel_tx.take().unwrap().send(());
                    //TODO handle err
                } else {
                    return Some(meta.script_name.clone());
                }
            }
        }
    }
    None
}

// TODO more efficient?
async fn clear_conflicting_for_script(
    state: &Arc<IglooState>,
    my_claims: &HashMap<EntityType, Vec<DeviceIDSelection>>,
) -> Option<String> {
    let mut state = state.scripts.states.lock().await;

    for (for_type, my_sels) in my_claims {
        for (_, meta) in &mut state.current {
            if let Some(their_sels) = meta.claims.get(for_type) {
                if DeviceIDSelection::any_collides_with_any(my_sels, their_sels) {
                    if meta.auto_cancel {
                        let _ = meta.cancel_tx.take().unwrap().send(());
                    } else {
                        return Some(meta.script_name.clone());
                    }
                }
            }
        }
    }

    None
}

#[derive(Error, Debug, Serialize)]
pub enum ScriptCancelFailure {
    #[error("not authorized")]
    NotAuthorized,
    #[error("not authorized for some of all")]
    PartiallyNotAuthorization,
    #[error("does not exist")]
    DoesNotExist,
}

/// Cancel all instances of a script
/// Returns if any attempts were not authorized
pub async fn cancel_all(
    state: &Arc<IglooState>,
    script_name: &str,
    uid: Option<usize>,
) -> Option<ScriptCancelFailure> {
    let mut state = state.scripts.states.lock().await;
    let mut not_authorized = false;
    for (_, meta) in &mut state.current {
        if meta.script_name == script_name {
            if uid.is_some() && !*meta.perms.get(uid.unwrap()).unwrap() {
                not_authorized = true
            } else {
                let _ = meta.cancel_tx.take().unwrap().send(());
            }
        }
    }
    match not_authorized {
        true => Some(ScriptCancelFailure::PartiallyNotAuthorization),
        false => None,
    }
}

/// Cancel instance of a script
pub async fn cancel(
    state: &Arc<IglooState>,
    id: u32,
    uid: Option<usize>,
) -> Option<ScriptCancelFailure> {
    let mut state = state.scripts.states.lock().await;
    if let Some(meta) = state.current.get_mut(&id) {
        if uid.is_some() && !*meta.perms.get(uid.unwrap()).unwrap() {
            return Some(ScriptCancelFailure::NotAuthorized);
        } else {
            let _ = meta.cancel_tx.take().unwrap().send(());
            return None;
        }
    }
    Some(ScriptCancelFailure::DoesNotExist)
}

fn claims_to_perms(auth: &Auth, claims: &HashMap<EntityType, Vec<DeviceIDSelection>>) -> BitVec {
    let mut bv = bitvec![1; auth.num_users];
    for sel in claims.values().flat_map(|v| v.iter()) {
        if matches!(sel, DeviceIDSelection::All) {
            continue;
        }

        bv &= auth.perms.get(sel.get_zid().unwrap()).unwrap();
    }
    bv
}
