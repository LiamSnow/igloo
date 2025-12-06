use crate::{
    Component,
    query::{DeviceSnapshot, ExtensionSnapshot, GroupSnapshot},
    web::dash::Dashboard,
};
use bincode::{Decode, Encode};
use derive_more::From;

/// WASM -> Igloo
#[derive(Debug, Clone, Encode, Decode, From)]
pub enum ClientMessage {
    // ExecSetQuery(SetQuery),
    Init,
    GetPageData(ClientPage),
}

#[derive(Debug, Clone, Encode, Decode, From)]
pub enum ClientPage {
    Dashboard(Option<String>),
    Tree,
    Settings,
    Penguin,
}

/// Igloo -> WASM
#[derive(Debug, Clone, Encode, Decode, From)]
pub enum ServerMessage {
    Dashboards(Vec<DashboardMeta>),
    Dashboard(Option<String>, Box<Dashboard>),
    Snapshot(Box<GlobalSnapshot>),
    ElementUpdate(ElementUpdate),
}

#[derive(Debug, Clone, Encode, Decode, From)]
pub struct DashboardMeta {
    pub id: String,
    pub display_name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ElementUpdate {
    pub watch_id: u32,
    pub value: Component,
}

#[derive(Debug, Clone, Encode, Decode, From)]
pub struct GlobalSnapshot {
    pub floes: Vec<ExtensionSnapshot>,
    pub groups: Vec<GroupSnapshot>,
    pub devices: Vec<DeviceSnapshot>,
}
