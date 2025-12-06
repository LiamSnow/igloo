use crate::{
    core::{IglooError, IglooRequest},
    query::QueryEngine,
    tree::DeviceTree,
};
use igloo_interface::{
    id::{DeviceID, FloeID, FloeRef, GenerationalID},
    ipc::IglooMessage,
};
use std::{error::Error, path::Path};
use tokio::fs;

pub mod handle;
pub use handle::*;

pub const FLOES_DIR: &str = "./floes";

pub async fn spawn_all(
    tree: &mut DeviceTree,
    engine: &mut QueryEngine,
    tx: &kanal::Sender<IglooRequest>,
) -> Result<(), Box<dyn Error>> {
    for id in get_all_floe_ids().await? {
        let (handle, writer) = FloeHandle::new(id.clone(), tx).await?;
        tree.attach_floe(engine, handle, writer)?;
    }
    Ok(())
}

/// Takes commands from a Floe and applies to Device Tree
pub fn handle_msg(
    tree: &mut DeviceTree,
    engine: &mut QueryEngine,
    fref: FloeRef,
    msg: IglooMessage,
) -> Result<(), IglooError> {
    use IglooMessage::*;
    match msg {
        CreateDevice(name) => {
            let id = tree.create_device(engine, name.clone(), fref)?;
            let floe = tree.floe(&fref)?;
            let mut scratch = Vec::with_capacity(name.len() + 32);
            let msg = IglooMessage::DeviceCreated(name, id.take());
            let res = floe.writer.try_write_immut(&msg, &mut scratch);
            if let Err(e) = res {
                eprintln!(
                    "{}/{fref}'s Unix socket is full. Killing.. Error={e}",
                    floe.id()
                );
                // TODO reboot instead of kill
                tree.detach_floe(engine, fref)?;
            }
            Ok(())
        }

        RegisterEntity {
            device,
            entity_name,
            entity_index,
        } => tree.register_entity(
            engine,
            DeviceID::from_comb(device),
            entity_name,
            entity_index,
        ),

        WriteComponents {
            device,
            entity,
            comps,
        } => tree.write_components(engine, DeviceID::from_comb(device), entity, comps),

        WhatsUpIgloo { .. } | DeviceCreated(..) | Custom { .. } => {
            // TODO return err
            Ok(())
        }
    }
}

async fn get_all_floe_ids() -> Result<Vec<FloeID>, IglooError> {
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
