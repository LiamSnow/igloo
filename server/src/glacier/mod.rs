use std::{error::Error, path::Path};

use igloo_interface::{
    CREATE_DEVICE, CreateDevice, DESELECT_ENTITY, DeviceCreated, END_TRANSACTION, REGISTER_ENTITY,
    RegisterEntity, SELECT_ENTITY, START_TRANSACTION, SelectEntity, StartTransaction, WRITE_INT,
    read_component,
};
use smallvec::SmallVec;
use tokio::{fs, sync::mpsc};

use crate::glacier::{
    entity::Entity,
    floe::FloeManager,
    query::{Query, handle_query},
    tree::{DeviceID, DeviceTree, FloeID, FloeRef},
};

mod entity;
mod floe;
pub mod query;
mod tree;

pub const FLOES_DIR: &str = "./floes";

pub type Commands = SmallVec<[Command; 6]>;

pub struct Command {
    pub cmd_id: u16,
    pub payload: Vec<u8>,
}

pub async fn spawn() -> Result<mpsc::Sender<Query>, Box<dyn Error>> {
    let mut tree = DeviceTree::load().await?;

    let (cmds_tx, cmds_rx) = mpsc::channel(100);
    let (query_tx, query_rx) = mpsc::channel(20);

    for name in get_all_floe_names().await? {
        let (reader, writer, max_supported_component) = floe::init(name.clone()).await?;

        let fid = FloeID(name);
        let fref = tree.attach_floe(fid.clone(), writer, max_supported_component)?;

        let cmds_tx_copy = cmds_tx.clone();
        tokio::spawn(async move {
            let man = FloeManager {
                fid,
                fref,
                cmds_tx: cmds_tx_copy,
                reader,
            };
            man.run().await;
        });
    }

    tokio::spawn(run(tree, cmds_rx, query_rx));

    Ok(query_tx)
}

async fn run(
    mut tree: DeviceTree,
    mut cmds_rx: mpsc::Receiver<(FloeRef, Commands)>,
    mut query_rx: mpsc::Receiver<Query>,
) {
    loop {
        tokio::select! {
            Some((fref, trans)) = cmds_rx.recv() => {
                if let Err(e) = handle_cmds(&mut tree, fref, trans).await {
                    eprintln!("Error handling commands from Floe #{fref:?}: {e}");
                }
            }

            Some(query) = query_rx.recv() => {
                if let Err(e) = handle_query(&mut tree, query).await {
                    eprintln!("Error handling query: {e}");
                }
            }
        }
    }
}

async fn handle_cmds(
    tree: &mut DeviceTree,
    fref: FloeRef,
    cmds: Commands,
) -> Result<(), Box<dyn Error>> {
    let mut trans = cmds.into_iter();
    let first = trans.next().unwrap();

    match first.cmd_id {
        START_TRANSACTION => {
            let params: StartTransaction = borsh::from_slice(&first.payload).unwrap();
            handle_trans(tree, fref, trans, DeviceID::from_comb(params.device_id)).await?;
        }
        CREATE_DEVICE => {
            let params: CreateDevice = borsh::from_slice(&first.payload)?;
            let new_id = tree.create_device(params.name.clone(), fref).await?;
            tree.floe_mut(fref)
                .writer
                .device_created(&DeviceCreated {
                    name: params.name,
                    id: new_id.take(),
                })
                .await?;
        }
        _ => {
            eprintln!("Floe #{fref:?} sent invalid command set (no start). Skipping..");
        }
    }

    Ok(())
}

async fn handle_trans(
    tree: &mut DeviceTree,
    fref: FloeRef,
    trans: smallvec::IntoIter<[Command; 6]>,
    did: DeviceID,
) -> Result<(), Box<dyn Error>> {
    let device = tree.device_mut(did).unwrap(); // FIXME unwrap

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
                        "Floe #{fref:?} malformed during a transaction with device ID={did}. Tried to write component without an entity selected.",
                    );
                }
            }
        }

        match line.cmd_id {
            REGISTER_ENTITY => {
                selected_entity = None;

                let params: RegisterEntity = borsh::from_slice(&line.payload).unwrap();
                if params.entity_idx as usize != device.entities.len() {
                    panic!(
                        "Floe #{fref:?} malformed during a transaction with device ID={did}. Tried to make entity idx={}, but should have been {}",
                        params.entity_idx,
                        device.entities.len()
                    );
                }

                device
                    .entity_idx_lut
                    .insert(params.entity_name, device.entities.len());
                device.entities.push(Entity::default());

                // TODO should this select entity?
            }

            SELECT_ENTITY => {
                let params: SelectEntity = borsh::from_slice(&line.payload).unwrap();
                let entity_idx = params.entity_idx as usize;
                if entity_idx > device.entities.len() - 1 {
                    panic!(
                        "Floe #{fref:?} malformed during a transaction with device ID={did}. Tried to select entity idx={entity_idx} which is not registered.",
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
                    "Floe #{fref:?} malformed during a transaction with device idx={did}. Sent unexpected command {cmd_id}",
                );
            }
        }
    }

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
