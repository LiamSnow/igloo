use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::From;

use crate::{
    Component,
    query::{DeviceSnapshot, FloeSnapshot, GroupSnapshot},
    web::dash::Dashboard,
};

/// WASM -> Igloo
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ClientMessage {
    // ExecSetQuery(SetQuery),
    Init,
    GetPageData(ClientPage),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ClientPage {
    Dashboard(Option<String>),
    Tree,
    Settings,
    Penguin,
}

/// Igloo -> WASM
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ServerMessage {
    Dashboards(Vec<DashboardMeta>),
    Dashboard(Option<String>, Box<Dashboard>),
    Snapshot(Box<GlobalSnapshot>),
    ElementUpdate(ElementUpdate),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, From)]
pub struct DashboardMeta {
    pub id: String,
    pub display_name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ElementUpdate {
    pub watch_id: u32,
    pub value: Component,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, From)]
pub struct GlobalSnapshot {
    pub floes: Vec<FloeSnapshot>,
    pub groups: Vec<GroupSnapshot>,
    pub devices: Vec<DeviceSnapshot>,
}
