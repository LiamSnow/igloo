use std::{error::Error, path::Path};

use igloo_interface::{
    Component, ComponentType, DESELECT_ENTITY, END_TRANSACTION, REGISTER_ENTITY, RegisterEntity,
    SELECT_ENTITY, START_DEVICE_TRANSACTION, START_REGISTRATION_TRANSACTION, SelectEntity,
    StartDeviceTransaction, StartRegistrationTransaction, WRITE_INT, read_component,
};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use tokio::{
    fs,
    sync::{mpsc, oneshot},
};

use crate::glacier::{
    entity::{Entity, HasComponent},
    query::{
        DeviceSnapshot, EntitySnapshot, FloeSnapshot, Query, QueryFilter, QueryKind, QueryTarget,
        Snapshot, ZoneSnapshot, handle_query,
    },
    tree::{Device, DeviceTree, Entities, Floe, Presense},
};

mod entity;
mod file;
mod floe;
pub mod query;
mod tree;

pub const FLOES_DIR: &str = "./floes";

pub type Transaction = SmallVec<[CommandLine; 6]>;

pub struct CommandLine {
    pub cmd_id: u16,
    pub payload: Vec<u8>,
}

pub async fn spawn() -> Result<mpsc::Sender<Query>, Box<dyn Error>> {
    let mut tree = DeviceTree::load().await?;

    let (trans_tx, trans_rx) = mpsc::channel(100);
    let (query_tx, query_rx) = mpsc::channel(20);

    for name in get_all_floe_names().await? {
        let (writer, max_supported_component) =
            floe::spawn(name.clone(), tree.floes.len() as u16, trans_tx.clone()).await?;

        tree.add_floe(
            name,
            Floe {
                writer,
                max_supported_component,
            },
        )?;
    }

    tokio::spawn(run(tree, trans_rx, query_rx));

    Ok(query_tx)
}

async fn run(
    mut tree: DeviceTree,
    mut trans_rx: mpsc::Receiver<(u16, Transaction)>,
    mut query_rx: mpsc::Receiver<Query>,
) {
    loop {
        tokio::select! {
            Some((floe_idx, trans)) = trans_rx.recv() => {
                handle_trans(&mut tree, floe_idx, trans).await;
            }

            Some(query) = query_rx.recv() => {
                handle_query(&mut tree, query).await;
            }
        }
    }
}

async fn handle_trans(tree: &mut DeviceTree, floe_idx: u16, trans: Transaction) {
    if trans.len() < 3 {
        eprintln!("Floe #{floe_idx} sent invalid transaction (too short). Skipping..");
        return;
    }

    let mut trans = trans.into_iter();
    let first = trans.next().unwrap();

    match first.cmd_id {
        START_REGISTRATION_TRANSACTION => {
            let params: StartRegistrationTransaction = borsh::from_slice(&first.payload).unwrap();
            let res = handle_registration_trans(tree, floe_idx, trans, params).await;
            if let Err(e) = res {
                eprintln!(
                    "Error handling device registration transaction from Floe #{floe_idx}: {e}"
                );
            }
        }
        START_DEVICE_TRANSACTION => {
            let params: StartDeviceTransaction = borsh::from_slice(&first.payload).unwrap();
            let res = handle_device_trans(tree, floe_idx, trans, params).await;
            if let Err(e) = res {
                eprintln!("Error handling device transaction from Floe #{floe_idx}: {e}");
            }
        }
        _ => {
            eprintln!("Floe #{floe_idx} sent invalid transaction (no start). Skipping..");
        }
    }
}

async fn handle_device_trans(
    tree: &mut DeviceTree,
    floe_idx: u16,
    trans: smallvec::IntoIter<[CommandLine; 6]>,
    params: StartDeviceTransaction,
) -> Result<(), Box<dyn Error>> {
    let devices = &mut tree.devices[floe_idx as usize];

    let device_idx = params.device_idx as usize;
    if device_idx > devices.len() - 1 {
        panic!(
            "Floe #{floe_idx} malformed. Tried to start device transaction with invalid device idx={}",
            params.device_idx
        );
    }

    let device = devices.get_mut(device_idx).unwrap();
    let mut selected_entity: Option<&mut Entity> = None;

    for line in trans {
        if line.cmd_id > WRITE_INT {
            match &mut selected_entity {
                Some(entity) => {
                    let val = read_component(line.cmd_id, line.payload).unwrap();
                    // set the entity, if we added
                    // something new, register it in presense
                    if let Some(comp_typ) = entity.set(val) {
                        device.presense.set(comp_typ);
                    }
                    continue;
                }
                None => {
                    panic!(
                        "Floe #{floe_idx} malformed during a transaction with device idx={device_idx}. Tried to write component without an entity selected.",
                    );
                }
            }
        }

        match line.cmd_id {
            SELECT_ENTITY => {
                let params: SelectEntity = borsh::from_slice(&line.payload).unwrap();
                let entity_idx = params.entity_idx as usize;
                if entity_idx > device.entities.len() - 1 {
                    panic!(
                        "Floe #{floe_idx} malformed during a transaction with device idx={device_idx}. Tried to select entity idx={entity_idx} which is not registered.",
                    );
                }
                selected_entity = Some(device.entities.get_mut(entity_idx).unwrap());
            }

            DESELECT_ENTITY => {
                selected_entity = None;
            }

            END_TRANSACTION => {
                break;
            }

            cmd_id => {
                panic!(
                    "Floe #{floe_idx} malformed during a transaction with device idx={device_idx}. Sent unexpected command {cmd_id}",
                );
            }
        }
    }

    Ok(())
}

async fn handle_registration_trans(
    tree: &mut DeviceTree,
    floe_idx: u16,
    trans: smallvec::IntoIter<[CommandLine; 6]>,
    params: StartRegistrationTransaction,
) -> Result<(), Box<dyn Error>> {
    let devices = &mut tree.devices[floe_idx as usize];

    if params.device_idx as usize != devices.len() {
        panic!(
            "Floe #{floe_idx} malformed. Tried to register new device under idx={} but should have been {}",
            params.device_idx,
            devices.len()
        );
    }

    let mut entities = Entities::default();
    let mut presense = Presense::default();
    let mut entity_idx_lut = FxHashMap::default();
    let mut selected_entity: Option<&mut Entity> = None;

    for line in trans {
        if line.cmd_id > WRITE_INT {
            match &mut selected_entity {
                Some(entity) => {
                    let val = read_component(line.cmd_id, line.payload).unwrap();
                    // set the entity, if we added
                    // something new, register it in presense
                    if let Some(comp_typ) = entity.set(val) {
                        presense.set(comp_typ);
                    }
                    continue;
                }
                None => {
                    panic!(
                        "Floe #{floe_idx} malformed when registering device '{}'. Tried to write component without an entity selected.",
                        params.initial_name
                    );
                }
            }
        }

        match line.cmd_id {
            REGISTER_ENTITY => {
                let rep: RegisterEntity = borsh::from_slice(&line.payload).unwrap();

                if rep.entity_idx as usize != entities.len() {
                    panic!(
                        "Floe #{floe_idx} malformed when registering device '{}'. Tried to register entity under idx={} but should have been {}",
                        params.initial_name,
                        rep.entity_idx,
                        entities.len(),
                    );
                }

                entity_idx_lut.insert(rep.entity_name, entities.len());
                selected_entity = None;
                entities.push(Entity::default());
                // TODO should this also select the entity?
            }

            SELECT_ENTITY => {
                let sep: SelectEntity = borsh::from_slice(&line.payload).unwrap();
                let entity_idx = sep.entity_idx as usize;
                if entity_idx > entities.len() - 1 {
                    panic!(
                        "Floe #{floe_idx} malformed when registering device '{}'. Tried to select entity idx={} which is not registered.",
                        params.initial_name, sep.entity_idx,
                    );
                }
                selected_entity = Some(entities.get_mut(entity_idx).unwrap());
            }

            DESELECT_ENTITY => {
                selected_entity = None;
            }

            END_TRANSACTION => {
                break;
            }

            cmd_id => {
                panic!(
                    "Floe #{floe_idx} malformed when registering device '{}'. Sent unexpected command {cmd_id}",
                    params.initial_name,
                );
            }
        }
    }

    let floe_id = tree.floe_id_lut[floe_idx as usize].clone();
    tree.register_device(
        floe_id,
        floe_idx,
        params.device_id,
        Device {
            initial_name: params.initial_name,
            entities,
            presense,
            entity_idx_lut,
        },
    )
    .await?;

    Ok(())
}

async fn get_all_floe_names() -> Result<Vec<String>, Box<dyn Error>> {
    let floes_path = Path::new(FLOES_DIR);
    if !floes_path.exists() {
        fs::create_dir(floes_path).await?;
        println!("Created directory: {FLOES_DIR}");
    } else if !floes_path.is_dir() {
        panic!("{FLOES_DIR} exists but is not a directory!");
    }

    let mut entries = fs::read_dir(floes_path).await?;
    let mut res = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let Some(name) = name.to_str() else {
            continue;
        };

        res.push(name.to_string());
    }

    Ok(res)
}
