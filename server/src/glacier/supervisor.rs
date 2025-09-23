use std::path::Path;

use tokio::{fs, sync::mpsc, task::JoinHandle};

use crate::glacier::{
    Glacier, GlacierError,
    floe::FloeHandle,
    tree::{DeviceTree, DeviceTreeUpdate},
};

pub struct GlacierSupervisor {
    floe_handles: Vec<FloeHandle>,
    glacier_handle: JoinHandle<()>,
    update_tx: mpsc::Sender<DeviceTreeUpdate>,
}

impl GlacierSupervisor {
    pub async fn new() -> Result<Self, GlacierError> {
        let state = DeviceTree::load().await.unwrap();

        let (update_tx, update_rx) = mpsc::channel::<DeviceTreeUpdate>(1000);

        let glacier = Glacier::new(update_rx, state);
        let glacier_handle = tokio::spawn(glacier.run());

        let mut floe_handles = Vec::new();

        let floes_path = Path::new("./floes");

        if !floes_path.exists() {
            fs::create_dir(floes_path).await?;
            println!("Created directory: ./floes");
        } else if !floes_path.is_dir() {
            panic!("./floes exists but is not a directory!");
        }

        let mut entries = fs::read_dir(floes_path).await?;

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

            let handle = FloeHandle::new(name, update_tx.clone()).await?;
            floe_handles.push(handle);
        }

        Ok(GlacierSupervisor {
            floe_handles,
            glacier_handle,
            update_tx,
        })
    }

    #[allow(dead_code)]
    pub async fn add_floe(&mut self, name: &str) -> Result<(), GlacierError> {
        let handle = FloeHandle::new(name, self.update_tx.clone()).await?;
        self.floe_handles.push(handle);
        Ok(())
    }

    pub async fn shutdown(self) {
        println!("Shutting down FloeManager...");

        drop(self.update_tx);

        for handle in self.floe_handles {
            println!("[{}] Shutting down floe", handle.name);
            handle.shutdown().await;
        }

        let _ = self.glacier_handle.await;

        println!("FloeManager shutdown complete");
    }
}
