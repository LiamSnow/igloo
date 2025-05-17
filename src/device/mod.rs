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
    ) -> (Self, Vec<oneshot::Sender<Arc<IglooState>>>) {
        let span = span!(Level::INFO, "Devices");
        let _enter = span.enter();
        info!("initializing");

        let mut istate_txs = Vec::new();
        let (on_change_tx, i) = tasks::init();
        istate_txs.push(i);

        let mut channels = Vec::with_capacity(lut.num_devs);
        for did in 0..lut.num_devs {
            let (cmd_tx, cmd_rx) = mpsc::channel::<TargetedEntityCommand>(5);
            let res = dev_cfgs.remove(0).spawn(
                did,
                dev_sels.remove(0),
                cmd_rx,
                on_change_tx.clone(),
            );
            if let Some(i) = res {
                istate_txs.push(i);
            }
            channels.push(cmd_tx);
        }

        (Self {
            channels,
            states: Mutex::new(vec![make_state_hashmap(); lut.num_devs]),
            lut,
        }, istate_txs)
    }
}

fn make_state_hashmap() -> HashMap<String, EntityState> {
    let mut h = HashMap::new();
    h.insert("connected".to_string(), EntityState::Connection(false));
    h
}
