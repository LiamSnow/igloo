use std::collections::HashMap;

use bitvec::{bitvec, vec::BitVec};

use crate::{config::AuthConfig, device::DeviceIDLut, selector::{Selection, SelectorError}};

#[derive(Default)]
pub struct Auth {
    pub num_users: usize,
    pub uid_lut: HashMap<String, usize>,
    pub gid_lut: HashMap<String, usize>,
    /// uid -> pw hash
    pub pw_hashes: Vec<String>,
    /// gid -> [uid]
    pub group: Vec<Vec<usize>>,
    /// zid,uid -> allowed
    pub perms: Vec<BitVec>,
}

impl Auth {
    pub fn init(
        cfg: AuthConfig,
        dev_ids: &DeviceIDLut
    ) -> Result<Self, SelectorError> {
        let mut auth = Auth::default();
        let mut all_uids = Vec::new();
        let (mut uid, mut gid) = (0, 1);

        for (username, user) in cfg.users {
            auth.uid_lut.insert(username, uid);
            auth.pw_hashes.push(user.password_hash);
            all_uids.push(uid);
            uid += 1;
        }
        auth.num_users = uid;

        auth.gid_lut.insert("all".to_string(), 0);
        auth.group.push(all_uids);

        for (name, users) in cfg.groups {
            auth.gid_lut.insert(name, gid);
            let mut uids = Vec::new();
            for user in users {
                let uid = auth.uid_lut.get(&user).unwrap(); //FIXME
                uids.push(*uid);
            }
            auth.group.push(uids);
            gid += 1;
        }

        // build lists of perms
        let mut all_perms: Option<BitVec> = None;
        let mut zone_perms: HashMap<usize, BitVec> = HashMap::new();
        for (sel_str, target) in cfg.permissions {
            let sel = Selection::from_str(&dev_ids, &sel_str)?;
            let uids = auth.get_uids(&target).unwrap(); //FIXME
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
        for zid in 0..dev_ids.num_zones {
            if let Some(bv) = zone_perms.get(&zid) {
                auth.perms.push(bv.clone());
            } else {
                auth.perms.push(all_perms.clone());
            }
        }

        Ok(auth)
    }

    /// if group name: returns all uids in group
    /// if user name: return their uid
    pub fn get_uids(&self, s: &str) -> Option<Vec<usize>> {
        if let Some(gid) = self.gid_lut.get(s) {
            Some(self.group.get(*gid).unwrap().clone())
        } else if let Some(uid) = self.uid_lut.get(s) {
            Some(vec![*uid])
        } else {
            None
        }
    }

    /// Checks whether a user has authorization to access a Selection
    pub fn is_authorized(&self, sel: &Selection, uid: usize) -> bool {
        if matches!(sel, Selection::All) {
            // calls to all will only apply to those they have permission for
            return true;
        }

        let zid = sel.get_zid().unwrap();
        *self.perms.get(zid).unwrap().get(uid).unwrap()
    }
}
