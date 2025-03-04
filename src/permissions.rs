use std::collections::HashMap;
use std::error::Error;

use bitvec::prelude::bitvec;
use bitvec::vec::BitVec;

use crate::auth::Auth;
use crate::map::DeviceIDLut;
use crate::selector::Selection;

#[derive(Default)]
pub struct Permissions {
    /// zid,uid -> allowed
    pub zone: Vec<BitVec>,
    // pub scripts: BitVec
}

impl Permissions {
    pub fn init(
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


    /// Checks whether a user has permission to access a Selection
    pub fn has_perm(&self, sel: &Selection, uid: usize) -> bool {
        if matches!(sel, Selection::All) {
            // calls to all will only apply to those they have permission for
            return true;
        }

        let zid = sel.get_zid().unwrap();
        *self.zone.get(zid).unwrap().get(uid).unwrap()
    }
}
