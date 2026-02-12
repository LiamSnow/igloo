use std::{collections::HashMap, net::SocketAddr, time::SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Auth {
    /// config versions simply increment by 1 for every breaking change
    /// it is required that `file.rs` contains migration code for every version
    pub(super) version: u32,
    /// UID -> User
    pub(super) users: HashMap<Uuid, User>,
    /// GID -> Group
    pub(super) groups: HashMap<Uuid, Group>,
    /// Token -> Session
    pub(super) sessions: HashMap<Uuid, Session>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub(super) username: String,
    // bcrypt hash string with all parts (salt, cost, password_hash)
    pub(super) hash_str: String,
    pub(super) display_name: String,
    pub(super) groups: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub(super) name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Session {
    pub(super) uid: Uuid,
    pub(super) addresses: Vec<SocketAddr>,
    pub(super) expires_at: SystemTime,
}

// -----

#[derive(Debug, Deserialize, Serialize)]
pub struct Permissions {
    pub(super) read: Vec<UserOrGroup>,
    pub(super) write: Vec<UserOrGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum UserOrGroup {
    User(Uuid),
    Group(Uuid),
}
