use std::{collections::HashMap, error::Error, sync::Arc};

use axum::extract::ws::Utf8Bytes;
use bitvec::prelude::bitvec;
use bitvec::vec::BitVec;
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::{
    command::TargetedSubdeviceCommand,
    config::{IglooConfig, User},
    effects::EffectsState,
    elements::Elements,
    providers::{esphome, DeviceConfig},
    selector::Selection,
};

pub struct IglooStack {
    /// did -> dev command channnel
    pub dev_chans: Vec<mpsc::Sender<TargetedSubdeviceCommand>>,
    pub dev_lut: DeviceIDLut,
    pub effects_state: Mutex<EffectsState>,
    pub elements: Elements,
    pub ws_broadcast: broadcast::Sender<Utf8Bytes>,
    pub auth: Auth,
    pub perms: Permissions,
    pub scripts: HashMap<String, Vec<String>>,
}

impl IglooStack {
    pub async fn init(icfg: IglooConfig) -> Result<Arc<Self>, Box<dyn Error>> {
        let (dev_lut, mut dev_cfgs, mut dev_sels) = DeviceIDLut::init(icfg.zones);

        let ws_broadcast = broadcast::Sender::new(20);

        let auth = Auth::init(icfg.users, icfg.user_groups);
        let perms = Permissions::init(icfg.permissions, &auth, &dev_lut)?;
        let scripts = icfg.scripts;
        let (elements, mut subscribers) = Elements::init(icfg.ui, &dev_lut, ws_broadcast.clone(), &perms, &auth)?;

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
            effects_state: Mutex::new(EffectsState::default()),
            ws_broadcast,
            auth,
            perms,
            scripts,
        }))
    }
}

#[derive(Default)]
pub struct Auth {
    pub num_users: usize,
    pub uid_lut: HashMap<String, usize>,
    pub gid_lut: HashMap<String, usize>,
    /// uid -> pw hash
    pub pw_hashes: Vec<String>,
    /// gid -> [uid]
    pub group: Vec<Vec<usize>>,
}

impl Auth {
    fn init(users: HashMap<String, User>, user_groups: HashMap<String, Vec<String>>) -> Self {
        let mut auth = Auth::default();
        let mut all_uids = Vec::new();
        let (mut uid, mut gid) = (0, 1);

        for (username, user) in users {
            auth.uid_lut.insert(username, uid);
            auth.pw_hashes.push(user.password_hash);
            all_uids.push(uid);
            uid += 1;
        }
        auth.num_users = uid;

        auth.gid_lut.insert("all".to_string(), 0);
        auth.group.push(all_uids);

        for (name, users) in user_groups {
            auth.gid_lut.insert(name, gid);
            let mut uids = Vec::new();
            for user in users {
                let uid = auth.uid_lut.get(&user).unwrap(); //FIXME
                uids.push(*uid);
            }
            auth.group.push(uids);
            gid += 1;
        }

        auth
    }

    pub fn lut(&self, s: &str) -> Option<Vec<usize>> {
        if let Some(gid) = self.gid_lut.get(s) {
            Some(self.group.get(*gid).unwrap().clone())
        } else if let Some(uid) = self.uid_lut.get(s) {
            Some(vec![*uid])
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct Permissions {
    /// zid,uid -> allowed
    pub zone: Vec<BitVec>,
    // pub scripts: BitVec
}

impl Permissions {
    fn init(
        cfg: HashMap<String, String>,
        auth: &Auth,
        dev_lut: &DeviceIDLut,
    ) -> Result<Self, Box<dyn Error>> {
        // build lists of perms
        let mut all_perms: Option<BitVec> = None;
        let mut zone_perms: HashMap<usize, BitVec> = HashMap::new();
        for (sel_str, target) in cfg {
            let sel = Selection::from_str(&dev_lut, &sel_str)?;
            let uids = auth.lut(&target).unwrap(); //FIXME
            let mut bv = bitvec![0; auth.num_users];
            for uid in uids {
                bv.set(uid, true);
            }
            match sel {
                Selection::All => all_perms = Some(bv),
                Selection::Zone(zid, _, _) => {
                    zone_perms.insert(zid, bv);
                }
                _ => println!("Permissions can only be applied to zones and all. Skipping."),
            }
        }

        //default to allow all users
        let all_perms = all_perms.unwrap_or(bitvec![1; auth.num_users]);

        // build perms by zid
        let mut perms = Permissions::default();
        for zid in 0..dev_lut.num_zones {
            if let Some(bv) = zone_perms.get(&zid) {
                perms.zone.push(bv.clone());
            } else {
                perms.zone.push(all_perms.clone());
            }
        }

        Ok(perms)
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
