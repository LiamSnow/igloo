use std::{collections::HashMap, error::Error, sync::Arc};

use axum::extract::ws::Utf8Bytes;
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::{
    auth::Auth,
    command::TargetedSubdeviceCommand,
    config::{IglooConfig, ScriptConfig},
    elements::Elements,
    permissions::Permissions,
    providers::{esphome, DeviceConfig}, scripts::ScriptStates,
};

pub struct IglooStack {
    /// did -> dev command channnel
    pub dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>>,
    pub dev_lut: DeviceIDLut,
    pub elements: Elements,
    pub ws_broadcast: broadcast::Sender<Utf8Bytes>,
    pub auth: Auth,
    pub perms: Permissions,
    pub script_states: Mutex<ScriptStates>,
    pub script_configs: HashMap<String, ScriptConfig>,
}

impl IglooStack {
    pub async fn init(icfg: IglooConfig) -> Result<Arc<Self>, Box<dyn Error>> {
        let (dev_lut, mut dev_cfgs, mut dev_sels) = DeviceIDLut::init(icfg.zones);

        let ws_broadcast = broadcast::Sender::new(20);

        let auth = Auth::init(icfg.users, icfg.user_groups);
        let perms = Permissions::init(icfg.permissions, &auth, &dev_lut)?;
        let (elements, mut subscribers) =
            Elements::init(icfg.ui, &dev_lut, ws_broadcast.clone(), &perms, &auth)?;

        // Make Devices
        let mut dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>> = Vec::new();
        for did in 0..dev_lut.num_devs {
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
            dev_lut,
            elements,
            ws_broadcast,
            auth,
            perms,
            script_states: Mutex::new(ScriptStates::default()),
            script_configs: icfg.scripts,
        }))
    }
}


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

impl DeviceIDLut {
    fn init(
        zones: HashMap<String, HashMap<String, DeviceConfig>>,
    ) -> (Self, Vec<DeviceConfig>, Vec<String>) {
        let (mut next_did, mut next_zid) = (0, 0);
        let mut lut = DeviceIDLut::default();
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
        lut.num_devs = next_did;
        lut.num_zones = next_zid;
        (lut, dev_cfgs, dev_sels)
    }
}
