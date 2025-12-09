use crate::{
    core::{ClientManager, IglooError, IglooRequest},
    query::QueryEngine,
    tree::DeviceTree,
};
use igloo_interface::{
    id::{DeviceID, EntityID, EntityIndex, ExtensionID, ExtensionIndex, GenerationalID},
    ipc::IglooMessage,
};
use std::{error::Error, path::Path};
use tokio::fs;

pub mod handle;
pub use handle::*;

pub const EXTS_DIR: &str = "./extensions";

pub async fn spawn_all(
    cm: &mut ClientManager,
    tree: &mut DeviceTree,
    engine: &mut QueryEngine,
    tx: &kanal::Sender<IglooRequest>,
) -> Result<(), Box<dyn Error>> {
    for id in get_all_ext_ids().await? {
        let (handle, writer) = ExtensionHandle::new(id.clone(), tx).await?;
        tree.attach_ext(cm, engine, handle, writer)?;
    }
    Ok(())
}

/// Takes commands from a Extension and applies to Device Tree
pub fn handle_msg(
    cm: &mut ClientManager,
    tree: &mut DeviceTree,
    engine: &mut QueryEngine,
    xindex: ExtensionIndex,
    msg: IglooMessage,
) -> Result<(), IglooError> {
    use IglooMessage::*;
    match msg {
        CreateDevice(name) => {
            let id = tree.create_device(cm, engine, name.clone(), xindex)?;
            let ext = tree.ext(&xindex)?;
            let mut scratch = Vec::with_capacity(name.len() + 32);
            let msg = IglooMessage::DeviceCreated(name, id.take());
            let res = ext.writer.try_write_immut(&msg, &mut scratch);
            if let Err(e) = res {
                eprintln!(
                    "{}/{xindex}'s Unix socket is full. Killing.. Error={e}",
                    ext.id()
                );
                // TODO reboot instead of kill
                tree.detach_ext(cm, engine, xindex)?;
            }
            Ok(())
        }

        RegisterEntity {
            device,
            entity_id,
            entity_index,
        } => tree.register_entity(
            cm,
            engine,
            DeviceID::from_comb(device),
            EntityID(entity_id),
            EntityIndex(entity_index),
        ),

        WriteComponents {
            device,
            entity,
            comps,
        } => tree.write_components(
            cm,
            engine,
            DeviceID::from_comb(device),
            EntityIndex(entity),
            comps,
        ),

        WhatsUpIgloo { .. } | DeviceCreated(..) | Custom { .. } => {
            // TODO return err
            Ok(())
        }
    }
}

async fn get_all_ext_ids() -> Result<Vec<ExtensionID>, IglooError> {
    let exts_path = Path::new(EXTS_DIR);
    if !exts_path.exists() {
        fs::create_dir(exts_path).await?;
        println!("Created directory: {EXTS_DIR}");
    } else if !exts_path.is_dir() {
        panic!("{EXTS_DIR} exists but is not a directory!");
    }

    let mut entries = fs::read_dir(exts_path).await?;
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

        res.push(ExtensionID(name.to_string()));
    }

    Ok(res)
}
