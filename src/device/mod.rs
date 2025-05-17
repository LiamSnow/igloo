pub mod ids;
pub mod selection_str;
pub mod error;
pub mod providers;
pub mod tasks;

use std::{collections::HashMap, sync::Arc};

use ids::DeviceIDLut;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{info, span, Level};
use providers::DeviceConfig;

use crate::{
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

        let (on_change_tx, back_cmd_tx) = tasks::init(istate_rx);

        let mut channels = Vec::with_capacity(lut.num_devs);
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
            states: Mutex::new(vec![make_state_hashmap(); lut.num_devs]),
            lut,
        }
    }
}

fn make_state_hashmap() -> HashMap<String, EntityState> {
    let mut h = HashMap::new();
    h.insert("connected".to_string(), EntityState::Connection(false));
    h
}
