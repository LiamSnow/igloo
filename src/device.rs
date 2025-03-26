use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc, Mutex};

use crate::{
    elements::{self, Elements},
    entity::{EntityState, TargetedEntityCommand},
    providers::{dummy, esphome, periodic, DeviceConfig},
};

#[derive(Default)]
pub struct DeviceIDLut {
    pub num_devs: usize,
    pub num_zones: usize,
    /// zid (zone ID) -> start_did, end_did
    pub did_range: Vec<(usize, usize)>,
    /// zid, dev_name -> did (device ID)
    pub did: Vec<HashMap<String, usize>>,
    /// zone_name -> zid
    pub zid: HashMap<String, usize>,
}

pub type DeviceChannels = Vec<mpsc::Sender<TargetedEntityCommand>>;

pub struct Devices {
    /// did -> dev command channnel
    pub channels: DeviceChannels,
    /// did, entity_name -> state
    pub states: Arc<Mutex<Vec<HashMap<String, EntityState>>>>,
    pub lut: DeviceIDLut,
}

impl Devices {
    pub fn init(
        lut: DeviceIDLut,
        mut dev_cfgs: Vec<DeviceConfig>,
        mut dev_sels: Vec<String>,
        elements: Arc<Elements>,
    ) -> Self {
        let mut channels = Vec::new();
        let (on_change_tx, on_change_rx) = mpsc::channel(10); //FIXME size?
        let mut states = Vec::new();
        for did in 0..lut.num_devs {
            let dev_sel = dev_sels.remove(0); //FIXME .get(did)?
            let dev_cfg = dev_cfgs.remove(0);
            let (cmd_tx, cmd_rx) = mpsc::channel::<TargetedEntityCommand>(5);

            match dev_cfg {
                DeviceConfig::ESPHome(cfg) => {
                    tokio::spawn(esphome::task(
                        cfg,
                        did,
                        dev_sel,
                        cmd_rx,
                        on_change_tx.clone(),
                    ));
                }
                DeviceConfig::Dummy(cfg) => {
                    tokio::spawn(dummy::task(cfg, did, dev_sel, cmd_rx, on_change_tx.clone()));
                }
                DeviceConfig::PeriodicTask(cfg) => {
                    tokio::spawn(periodic::task(
                        cfg,
                        did,
                        dev_sel,
                        cmd_rx,
                        on_change_tx.clone(),
                    ));
                }
                DeviceConfig::MQTT(_cfg) => todo!(),
            }

            channels.push(cmd_tx);
            states.push(HashMap::new());
        }

        let states = Arc::new(Mutex::new(states));
        tokio::spawn(state_task(states.clone(), on_change_rx, elements.clone()));

        Self {
            channels,
            states,
            lut,
        }
    }
}

impl DeviceIDLut {
    pub fn init(
        devices: HashMap<String, HashMap<String, DeviceConfig>>,
    ) -> (Self, Vec<DeviceConfig>, Vec<String>) {
        //make lut
        let (mut next_did, mut next_zid) = (0, 0);
        let mut lut = DeviceIDLut::default();
        let (mut dev_cfgs, mut dev_sels) = (Vec::new(), Vec::new());
        for (zone_name, devs) in devices {
            let start_did = next_did;
            let mut did_lut = HashMap::new();
            for (dev_name, dev_cfg) in devs {
                did_lut.insert(dev_name.clone(), next_did);
                dev_cfgs.push(dev_cfg);
                dev_sels.push(format!("{zone_name}.{dev_name}"));
                next_did += 1;
            }
            lut.did.push(did_lut);
            lut.did_range.push((start_did, next_did - 1));
            lut.zid.insert(zone_name, next_zid);
            next_zid += 1;
        }
        lut.num_devs = next_did;
        lut.num_zones = next_zid;
        (lut, dev_cfgs, dev_sels)
    }
}

async fn state_task(
    dev_states: Arc<Mutex<Vec<HashMap<String, EntityState>>>>,
    mut on_change_rx: mpsc::Receiver<(usize, String, EntityState)>,
    elements: Arc<Elements>,
) {
    //TODO group changes?
    while let Some((did, entity_name, value)) = on_change_rx.recv().await {
        //update elements
        elements::on_device_update(&dev_states, &elements, did, &entity_name, &value).await;

        //push to states
        let mut states = dev_states.lock().await;
        states[did].insert(entity_name, value.clone());
    }
}
