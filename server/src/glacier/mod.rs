use crate::glacier::{
    floe::FloeManager,
    query::{QueryEngine, QueryEngineRx, QueryEngineTx},
    tree::DeviceTree,
};
use igloo_interface::{
    CREATE_DEVICE, CreateDevice, DESELECT_ENTITY, DeviceCreated, END_TRANSACTION, REGISTER_ENTITY,
    RegisterEntity, SELECT_ENTITY, START_TRANSACTION, SelectEntity, StartTransaction,
    WRITE_INTEGER,
    id::{DeviceID, FloeID, FloeRef},
    read_component,
};
use smallvec::SmallVec;
use std::{error::Error, path::Path};
use tokio::{fs, sync::mpsc};

mod floe;
pub mod query;
pub mod tree;

pub const FLOES_DIR: &str = "./floes";

pub type Commands = SmallVec<[Command; 6]>;

pub struct Command {
    pub cmd_id: u16,
    pub payload: Vec<u8>,
}

pub async fn spawn() -> Result<QueryEngineTx, Box<dyn Error>> {
    let mut tree = DeviceTree::load().await?;
    let mut engine = QueryEngine::new();

    let (cmds_tx, cmds_rx) = mpsc::channel(100);
    let (query_tx, query_rx) = mpsc::channel(20);

    for fid in get_all_floe_names().await? {
        let (reader, writer, max_supported_component) = floe::init(fid.clone()).await?;

        tree.attach_floe(&mut engine, fid.clone(), writer, max_supported_component)
            .await?;

        let fref = *tree.floe_ref(&fid)?;

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

    tokio::spawn(run(tree, engine, cmds_rx, query_rx));

    Ok(query_tx)
}

async fn run(
    mut tree: DeviceTree,
    mut engine: QueryEngine,
    mut cmds_rx: mpsc::Receiver<(FloeRef, Commands)>,
    mut query_rx: QueryEngineRx,
) {
    loop {
        tokio::select! {
            Some((fref, trans)) = cmds_rx.recv() => {
                if let Err(e) = handle_cmds(&mut tree, &mut engine, fref, trans).await {
                    eprintln!("Error handling commands from Floe #{fref:?}: {e}");
                }
            }

            Some((query, tx)) = query_rx.recv() => {
                if let Err(e) = engine.evaluate(&mut tree, query, tx).await {
                    eprintln!("Error executing query: {e}");
                }
            }
        }
    }
}

// TODO probably move this stuff to tree/
async fn handle_cmds(
    tree: &mut DeviceTree,
    engine: &mut QueryEngine,
    fref: FloeRef,
    cmds: Commands,
) -> Result<(), Box<dyn Error>> {
    let mut trans = cmds.into_iter();
    let first = trans.next().unwrap();

    match first.cmd_id {
        START_TRANSACTION => {
            let params: StartTransaction = borsh::from_slice(&first.payload).unwrap();
            handle_trans(
                tree,
                engine,
                fref,
                trans,
                DeviceID::from_comb(params.device_id),
            )
            .await?;
        }
        CREATE_DEVICE => {
            let params: CreateDevice = borsh::from_slice(&first.payload)?;
            let new_id = tree
                .create_device(engine, params.name.clone(), fref)
                .await?;
            let floe = tree.floe_mut(&fref)?;
            floe.writer
                .device_created(&DeviceCreated {
                    name: params.name,
                    id: new_id.take(),
                })
                .await?;
            floe.writer.flush().await?;
        }
        _ => {
            eprintln!("Floe #{fref:?} sent invalid command set (no start). Skipping..");
        }
    }

    Ok(())
}

async fn handle_trans(
    tree: &mut DeviceTree,
    engine: &mut QueryEngine,
    fref: FloeRef,
    trans: smallvec::IntoIter<[Command; 6]>,
    did: DeviceID,
) -> Result<(), Box<dyn Error>> {
    let device = tree.device_mut(&did).unwrap(); // FIXME unwrap

    let mut selected_entity: Option<usize> = None;

    for line in trans {
        if line.cmd_id > WRITE_INTEGER {
            match selected_entity {
                Some(eidx) => {
                    let val = read_component(line.cmd_id, line.payload).unwrap();

                    DeviceTree::write_component(engine, device, did, eidx, val).await?;

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

                DeviceTree::register_entity(
                    engine,
                    device,
                    did,
                    params.entity_name,
                    params.entity_idx as usize,
                )
                .await?;

                // TODO should this select entity?
            }

            SELECT_ENTITY => {
                let params: SelectEntity = borsh::from_slice(&line.payload).unwrap();
                let eidx = params.entity_idx as usize;
                if eidx > device.num_entities() - 1 {
                    panic!(
                        "Floe #{fref:?} malformed during a transaction with device ID={did}. Tried to select entity idx={eidx} which is not registered.",
                    );
                }
                selected_entity = Some(eidx);
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

async fn get_all_floe_names() -> Result<Vec<FloeID>, Box<dyn Error>> {
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

        res.push(FloeID(name.to_string()));
    }

    Ok(res)
}
