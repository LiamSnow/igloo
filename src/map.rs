use std::{collections::HashMap, error::Error, sync::Arc};

use axum::extract::ws::Utf8Bytes;
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::{
    command::TargetedSubdeviceCommand,
    config::IglooConfig,
    effects::EffectsState,
    elements::Elements,
    providers::{esphome, DeviceConfig},
};

#[derive(Default)]
pub struct IDLut {
    /// zid (zone ID) -> start_did, end_did
    pub did_range: Vec<(usize, usize)>,
    /// zid, dev_name -> did (device ID)
    pub did: Vec<HashMap<String, usize>>,
    /// zone_name -> zid
    pub zid: HashMap<String, usize>,
}

// #[derive(Default)]
// pub struct PermissionLUT {
//     pub
// }

pub struct IglooStack {
    /// did -> dev command channnel
    pub dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>>,
    pub lut: IDLut,
    pub effects_state: Mutex<EffectsState>,
    pub elements: Elements,
    pub ws_broadcast: broadcast::Sender<Utf8Bytes>,
    //permissions zid,uid,allowed
    // pub zone_perms: Vec<Vec<bool>>,
}

impl IglooStack {
    pub async fn init(config: IglooConfig) -> Result<Arc<Self>, Box<dyn Error>> {
        let (lut, mut dev_cfgs, mut dev_sels) = IDLut::init(config.zones);

        let ws_broadcast = broadcast::Sender::new(20);

        let (elements, mut subscribers) = Elements::init(
            config.ui,
            &lut,
            ws_broadcast.clone(),
        )?;

        // Make Devices
        let mut dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>> = Vec::new();
        for did in 0..lut.did.len() {
            let dev_sel = dev_sels.remove(0);
            let dev_cfg = dev_cfgs.remove(0);
            let on_update_of_type = subscribers.of_types.remove(0);
            let on_subdev_update = subscribers.subdev.remove(0);
            let (cmd_tx, cmd_rx) = mpsc::channel::<TargetedSubdeviceCommand>(5);

            let task = match dev_cfg {
                DeviceConfig::ESPHome(cfg) => esphome::task(
                    cfg,
                    did,
                    dev_sel,
                    cmd_rx,
                    on_update_of_type,
                    on_subdev_update,
                ),
                DeviceConfig::HomeKit(_cfg) => todo!(),
            };
            tokio::spawn(task);

            dev_chans.push(cmd_tx);
        }

        Ok(Arc::new(IglooStack {
            dev_chans,
            lut,
            elements,
            effects_state: Mutex::new(EffectsState::default()),
            ws_broadcast,
        }))
    }
}

impl IDLut {
    pub fn init(zones: HashMap<String, HashMap<String, DeviceConfig>>) -> (Self, Vec<DeviceConfig>, Vec<String>) {
        let (mut next_did, mut next_zid) = (0, 0);
        let mut lut = IDLut::default();
        let (mut dev_cfgs, mut dev_sels) = (Vec::new(), Vec::new());
        for (zone_name, devs) in zones {
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

        (lut, dev_cfgs, dev_sels)
    }
}
