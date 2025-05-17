pub mod ids;
pub mod selection_str;
pub mod error;
pub mod providers;

use std::{collections::HashMap, sync::Arc};

use ids::DeviceIDLut;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info, span, Level};
use providers::DeviceConfig;

use crate::{
    cli::model::Cli,
    elements,
    entity::{EntityState, TargetedEntityCommand},
    state::IglooState,
};

pub type DeviceChannels = Vec<mpsc::Sender<TargetedEntityCommand>>;

pub struct Devices {
    /// did -> dev command channnel
    pub channels: DeviceChannels,
    /// did, entity_name -> state
    pub states: Mutex<Vec<HashMap<String, EntityState>>>,
    pub lut: DeviceIDLut,
}

impl Devices {
    pub fn init(
        lut: DeviceIDLut,
        mut dev_cfgs: Vec<DeviceConfig>,
        mut dev_sels: Vec<String>,
        istate_rx: oneshot::Receiver<Arc<IglooState>>,
    ) -> Self {
        let span = span!(Level::INFO, "Devices");
        let _enter = span.enter();
        info!("initializing");

        let (on_change_tx, on_change_rx) = mpsc::channel(10); //FIXME size?
        let (back_cmd_tx, back_cmd_rx) = mpsc::channel::<Cli>(5);
        tokio::spawn(async move {
            let istate = istate_rx.await.unwrap();
            tokio::spawn(state_task(on_change_rx, istate.clone()));
            tokio::spawn(back_cmd_task(back_cmd_rx, istate));
        });

        let mut channels = Vec::new();

        for did in 0..lut.num_devs {
            let (cmd_tx, cmd_rx) = mpsc::channel::<TargetedEntityCommand>(5);
            dev_cfgs.remove(0).spawn(
                did,
                dev_sels.remove(0),
                back_cmd_tx.clone(),
                cmd_rx,
                on_change_tx.clone(),
            );
            channels.push(cmd_tx);
        }

        Self {
            channels,
            states: Mutex::new(vec![HashMap::new(); lut.num_devs]),
            lut,
        }
    }
}


async fn state_task(
    mut on_change_rx: mpsc::Receiver<(usize, String, EntityState)>,
    istate: Arc<IglooState>,
) {
    let span = span!(Level::INFO, "Devices State Update Task");
    let _enter = span.enter();
    info!("running");

    //TODO group changes?
    while let Some((did, entity_name, value)) = on_change_rx.recv().await {
        //push to states
        {
            let mut states = istate.devices.states.lock().await;
            states[did].insert(entity_name.clone(), value.clone());
        }

        //update elements
        elements::state::on_device_update(&istate, did, &entity_name, &value).await;
    }
}

async fn back_cmd_task(
    mut back_cmd_rx: mpsc::Receiver<Cli>,
    istate: Arc<IglooState>,
) {
    let span = span!(Level::INFO, "Devices Back Command Task");
    let _enter = span.enter();
    info!("running");

    while let Some(cmd) = back_cmd_rx.recv().await {
        if let Err(e) = cmd.dispatch(&istate, None, true).await {
            error!("{}", serde_json::to_string(&e).unwrap());
        }
    }
}
