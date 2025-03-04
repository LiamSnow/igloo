use std::{collections::HashMap, sync::Arc};

use bitvec::prelude::bitvec;
use bitvec::vec::BitVec;
use error::ScriptError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::oneshot;

use crate::{
    command::SubdeviceType,
    config::ScriptConfig,
    map::{DeviceIDLut, IglooStack},
    selector::Selection,
};

mod basic;
mod builtin;
pub mod error;
mod python;

#[derive(Default)]
pub struct ScriptStates {
    pub next_id: u32,
    pub current: HashMap<u32, RunningScriptMeta>,
}

#[derive(Deserialize)]
pub struct ScriptMeta {
    pub claims: HashMap<SubdeviceType, Vec<String>>,
    pub auto_cancel: bool,
}

#[derive(Serialize)]
pub struct RunningScriptMeta {
    script_name: String,
    #[serde(skip_serializing)]
    claims: HashMap<SubdeviceType, Vec<Selection>>,
    auto_cancel: bool,
    #[serde(skip_serializing)]
    cancel_tx: Option<oneshot::Sender<()>>,
    #[serde(skip_serializing)]
    perms: BitVec,
}

pub async fn spawn(
    stack: &Arc<IglooStack>,
    script_name: String,
    extra_args: Vec<String>,
    uid: usize,
) -> Result<(), ScriptError> {
    if let Some(meta) = builtin::get_meta().get(&script_name) {
        let (id, cancel_rx) = init_script(stack, uid, &script_name, &extra_args, meta).await?;

        let res = builtin::spawn(&script_name, id, stack.clone(), uid, extra_args, cancel_rx).await;
        if let Err(err) = res {
            //TODO log
            return Err(ScriptError::BuiltInFailure(err.to_string()));
        }

        Ok(())
    } else if let Some(cfg) = stack.script_configs.get(&script_name) {
        let meta = cfg.get_meta();
        let (id, cancel_rx) = init_script(stack, uid, &script_name, &extra_args, meta).await?;

        match cfg {
            ScriptConfig::Python(cfg) => python::spawn(
                script_name,
                id,
                stack.clone(),
                uid,
                extra_args,
                cancel_rx,
                cfg.file.clone(),
            ),
            ScriptConfig::Basic(cfg) => basic::spawn(
                script_name,
                id,
                stack.clone(),
                uid,
                extra_args,
                cancel_rx,
                cfg.body.clone(),
            ),
        }

        Ok(())
    } else {
        Err(ScriptError::UnknownScript(script_name))
    }
}

async fn init_script(
    stack: &Arc<IglooStack>,
    uid: usize,
    script_name: &str,
    extra_args: &Vec<String>,
    meta: &ScriptMeta,
) -> Result<(u32, oneshot::Receiver<()>), ScriptError> {
    let claims = parse_claims(&stack.dev_lut, &meta.claims, &extra_args)?;

    let perms = calc_perms(&stack, &claims);
    if !*perms.get(uid).unwrap() {
        return Err(ScriptError::NotAuthorized);
    }

    let res = clear_conflicting_for_script(stack, &claims).await;
    if let Some(scr) = res {
        return Err(ScriptError::CouldNotCancel(scr));
    }

    let (cancel_tx, cancel_rx) = oneshot::channel();

    //push to stack
    let mut states = stack.script_states.lock().await;
    let id = states.next_id;
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
    states.next_id += 1;
    Ok((id, cancel_rx))
}

/// Parses String claims into Selections.
/// Replaces positional args (IE $1) with their values
fn parse_claims(
    dev_lut: &DeviceIDLut,
    raw: &HashMap<SubdeviceType, Vec<String>>,
    extra_args: &Vec<String>,
) -> Result<HashMap<SubdeviceType, Vec<Selection>>, ScriptError> {
    let mut res = HashMap::new();
    for (subdev_type, sel_strs) in raw {
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

            v.push(Selection::from_str(dev_lut, sel_str)?);
        }
        res.insert(subdev_type.clone(), v);
    }
    Ok(res)
}

/// Tries to stop scripts that currently claim ownership over the same device(s)
/// Returns Some(scripe_name) if that script is conflicting and cannot be cancelled
pub async fn clear_conflicting_for_cmd(
    stack: &Arc<IglooStack>,
    selection: &Selection,
    for_type: &SubdeviceType,
) -> Option<String> {
    let mut state = stack.script_states.lock().await;
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
    stack: &Arc<IglooStack>,
    my_claims: &HashMap<SubdeviceType, Vec<Selection>>,
) -> Option<String> {
    let mut state = stack.script_states.lock().await;

    for (for_type, my_sels) in my_claims {
        for (_, meta) in &mut state.current {
            if let Some(their_sels) = meta.claims.get(for_type) {
                if Selection::any_collides_with_any(my_sels, their_sels) {
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
    stack: &Arc<IglooStack>,
    script_name: &str,
    uid: usize,
) -> Option<ScriptCancelFailure> {
    //TODO permissions
    let mut state = stack.script_states.lock().await;
    let mut not_authorized = false;
    for (_, meta) in &mut state.current {
        if meta.script_name == script_name {
            if *meta.perms.get(uid).unwrap() {
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
pub async fn cancel(stack: &Arc<IglooStack>, id: u32, uid: usize) -> Option<ScriptCancelFailure> {
    //TODO permissions
    let mut state = stack.script_states.lock().await;
    if let Some(meta) = state.current.get_mut(&id) {
        if *meta.perms.get(uid).unwrap() {
            let _ = meta.cancel_tx.take().unwrap().send(());
            return None;
        } else {
            return Some(ScriptCancelFailure::NotAuthorized);
        }
    }
    Some(ScriptCancelFailure::DoesNotExist)
}

fn calc_perms(stack: &IglooStack, claims: &HashMap<SubdeviceType, Vec<Selection>>) -> BitVec {
    let mut bv = bitvec![1; stack.auth.num_users];
    for sel in claims.values().flat_map(|v| v.iter()) {
        if matches!(sel, Selection::All) {
            continue;
        }

        bv &= stack.perms.zone.get(sel.get_zid().unwrap()).unwrap();
    }
    bv
}
