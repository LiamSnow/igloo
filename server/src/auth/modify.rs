use std::{collections::HashMap, net::SocketAddr, time::SystemTime};

use bcrypt::{BcryptError, BcryptResult};
use uuid::Uuid;

use crate::auth::{
    CONFIG_VERSION, SESSION_DURATION,
    model::{Auth, Group, Session, User},
};

impl Auth {
    pub fn new() -> Self {
        Self {
            version: CONFIG_VERSION,
            users: HashMap::new(),
            groups: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, username: String, password: String) -> Result<Uuid, BcryptError> {
        let uid = Uuid::now_v7();

        if self.users.contains_key(&uid) {
            return self.add_user(username, password);
        }

        let hash_str = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

        self.users.insert(
            uid,
            User {
                username: username.clone(),
                display_name: username.clone(),
                hash_str,
                groups: vec![],
            },
        );

        Ok(uid)
    }

    pub fn add_group(&mut self, name: String) -> Uuid {
        let gid = Uuid::now_v7();

        if self.groups.contains_key(&gid) {
            return self.add_group(name);
        }

        self.groups.insert(gid, Group { name });

        gid
    }

    pub fn remove_user(&mut self, uid: &Uuid) -> Option<User> {
        self.users.remove(uid)
    }

    pub fn remove_group(&mut self, gid: &Uuid) -> Option<Group> {
        self.groups.remove(gid)
    }

    pub fn remove_session(&mut self, token: &Uuid) -> Option<Session> {
        self.sessions.remove(token)
    }

    pub fn get_all_sessions(&self, uid: Uuid) -> Vec<(&Uuid, &Session)> {
        let mut res = Vec::new();
        for (token, sesh) in &self.sessions {
            if sesh.uid == uid {
                res.push((token, sesh));
            }
        }
        res
    }

    /// removes all sessions for a user, returns # of sessions signed out
    pub fn remove_all_sessions(&mut self, uid: Uuid) -> usize {
        let mut tokens = Vec::new();

        for (token, sesh) in &self.sessions {
            if sesh.uid == uid {
                tokens.push(*token);
            }
        }

        for token in &tokens {
            self.sessions.remove(token);
        }

        tokens.len()
    }

    pub fn add_session(&mut self, uid: Uuid, address: SocketAddr) -> Uuid {
        let token = Uuid::now_v7();

        if self.sessions.contains_key(&token) {
            return self.add_session(uid, address);
        }

        self.sessions.insert(
            token,
            Session {
                uid,
                addresses: vec![address],
                expires_at: SystemTime::now() + SESSION_DURATION,
            },
        );

        token
    }

    /// logs in with a session token, returns Some(uid) if successful
    pub fn try_login_with_session(&mut self, token: Uuid, address: SocketAddr) -> Option<Uuid> {
        // session token doesn't exist
        if !self.sessions.contains_key(&token) {
            return None;
        }

        let sesh = self.sessions.get_mut(&token).unwrap();

        // session is expired
        if SystemTime::now() > sesh.expires_at {
            // remove it so we don't have to check again
            self.remove_session(&token);
            return None;
        }

        // session is valid -> log if IP is new
        if !sesh.addresses.contains(&address) {
            // TODO log, this could be a security risk
            sesh.addresses.push(address);
        }

        Some(sesh.uid)
    }

    pub fn lookup_username(&self, username: &str) -> Option<(Uuid, &User)> {
        for (uid, user) in &self.users {
            if user.username == username {
                return Some((*uid, user));
            }
        }
        None
    }

    pub fn try_login(&self, username: &str, password: &str) -> BcryptResult<Option<Uuid>> {
        let (uid, user) = match self.lookup_username(username) {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(match bcrypt::verify(password, &user.hash_str)? {
            true => Some(uid),
            false => None,
        })
    }
}
