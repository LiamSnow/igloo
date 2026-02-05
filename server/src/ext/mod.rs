use crate::{
    PACKAGES_DIR,
    core::{ClientManager, IglooError, IglooRequest},
    query::QueryEngine,
    tree::DeviceTree,
};
use igloo_interface::id::ExtensionID;
use std::error::Error;
use tokio::fs;

pub mod handle;
pub use handle::*;

pub const EXTS_DIR: &str = "extensions";

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

async fn get_all_ext_ids() -> Result<Vec<ExtensionID>, IglooError> {
    let mut exts_path = PACKAGES_DIR.get().unwrap().clone();
    exts_path.push(EXTS_DIR);

    if !exts_path.exists() {
        fs::create_dir(&exts_path).await?;
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
