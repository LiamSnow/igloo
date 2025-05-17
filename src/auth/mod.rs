use std::collections::HashMap;

use bitvec::{bitvec, vec::BitVec};
use error::AuthError;
use login::TokenDatabase;
use tracing::{info, span, Level};

use crate::{config::AuthConfig, device::ids::DeviceIDLut, device::ids::DeviceIDSelection};

pub mod error;
pub mod login;

pub struct Auth {
    pub num_users: usize,
    pub uid_lut: HashMap<String, usize>,
    pub gid_lut: HashMap<String, usize>,
    /// uid -> pw hash
    pub pw_hashes: Vec<String>,
    /// gid -> [uid]
    pub groups: Vec<Vec<usize>>,
    /// zid,uid -> allowed
    pub perms: Vec<BitVec>,
    pub token_db: TokenDatabase,
}

impl Auth {
    pub async fn init(cfg: AuthConfig, dev_ids: &DeviceIDLut) -> Result<Self, AuthError> {
        let span = span!(Level::INFO, "Auth");
        let _enter = span.enter();
        info!("initializing");

        let mut uid_lut = HashMap::new();
        let (mut all_uids, mut pw_hashes) = (Vec::new(), Vec::new());
        let mut uid = 0;
        for (username, user) in cfg.users {
            uid_lut.insert(username, uid);
            pw_hashes.push(user.password_hash);
            all_uids.push(uid);
            uid += 1;
        }
        let num_users = uid;

        let mut gid = 0;
        let mut gid_lut = HashMap::new();
        let mut groups = Vec::new();
        gid_lut.insert("all".to_string(), 0);
        groups.push(all_uids);
        for (name, users) in cfg.groups {
            gid_lut.insert(name, gid);
            let mut uids = Vec::new();
            for user in users {
                let uid = uid_lut.get(&user).unwrap(); //FIXME
                uids.push(*uid);
            }
            groups.push(uids);
            gid += 1;
        }

        // build lists of perms
        let mut all_perms: Option<BitVec> = None;
        let mut zone_perms: HashMap<usize, BitVec> = HashMap::new();
        for (sel_str, target) in cfg.permissions {
            let sel = DeviceIDSelection::from_str(&dev_ids, &sel_str)?;
            let uids = Self::get_uids_internal(&target, &uid_lut, &gid_lut, &groups).unwrap(); //FIXME
            let mut bv = bitvec![0; num_users];
            for uid in uids {
                bv.set(uid, true);
            }
            match sel {
                DeviceIDSelection::All => all_perms = Some(bv),
                DeviceIDSelection::Zone(zid, _, _) => {
                    zone_perms.insert(zid, bv);
                }
                _ => info!("Permissions can only be applied to zones and all. Skipping."),
            }
        }

        //default to allow all users
        let all_perms = all_perms.unwrap_or(bitvec![1; num_users]);

        // build perms by zid
        let mut perms = Vec::new();
        for zid in 0..dev_ids.num_zones {
            if let Some(bv) = zone_perms.get(&zid) {
                perms.push(bv.clone());
            } else {
                perms.push(all_perms.clone());
            }
        }

        Ok(Auth {
            num_users,
            uid_lut,
            gid_lut,
            pw_hashes,
            groups,
            perms,
            token_db: TokenDatabase::connect().await?,
        })
    }

    fn get_uids_internal(
        s: &str,
        uid_lut: &HashMap<String, usize>,
        gid_lut: &HashMap<String, usize>,
        group: &Vec<Vec<usize>>,
    ) -> Option<Vec<usize>> {
        if let Some(gid) = gid_lut.get(s) {
            Some(group.get(*gid).unwrap().clone())
        } else if let Some(uid) = uid_lut.get(s) {
            Some(vec![*uid])
        } else {
            None
        }
    }

    /// if group name: returns all uids in group
    /// if user name: return their uid
    pub fn get_uids(&self, s: &str) -> Option<Vec<usize>> {
        Self::get_uids_internal(s, &self.uid_lut, &self.gid_lut, &self.groups)
    }

    /// Checks whether a user has authorization to access a Selection
    pub fn is_authorized(&self, sel: &DeviceIDSelection, uid: usize) -> bool {
        if matches!(sel, DeviceIDSelection::All) {
            // calls to all will only apply to those they have permission for
            return true;
        }

        let zid = sel.get_zid().unwrap();
        *self.perms.get(zid).unwrap().get(uid).unwrap()
    }
}
