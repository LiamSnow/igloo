use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::From;

use crate::{Component, SetQuery, Snapshot, dash::Dashboard};

/// WASM -> Igloo
#[derive(Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ClientMessage {
    ExecSetQuery(SetQuery),
    Init,
    GetPageData(ClientPage),
}

#[derive(Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ClientPage {
    Dashboard(Option<String>),
    Tree,
    Settings,
    Penguin,
}

/// Igloo -> WASM
#[derive(Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ServerMessage {
    Dashboards(Vec<DashboardMeta>),
    Dashboard(Option<String>, Box<Dashboard>),
    Snapshot(Box<Snapshot>),
    ElementUpdate(ElementUpdate),
}

#[derive(Clone, BorshSerialize, BorshDeserialize, From)]
pub struct DashboardMeta {
    pub id: String,
    pub display_name: String,
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct ElementUpdate {
    pub watch_id: u32,
    pub value: Component,
}
