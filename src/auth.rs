use std::collections::HashMap;

use crate::config::UserConfig;

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
    pub fn init(user_cfgs: HashMap<String, UserConfig>, user_groups: HashMap<String, Vec<String>>) -> Self {
        let mut auth = Auth::default();
        let mut all_uids = Vec::new();
        let (mut uid, mut gid) = (0, 1);

        for (username, user) in user_cfgs {
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
