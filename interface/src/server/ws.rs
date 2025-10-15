use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::From;

use crate::{Component, SetQuery, dash::Dashboard};

/// WASM -> Igloo
#[derive(Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ClientMessage {
    ExecSetQuery(SetQuery),
    /// Tells Igloo what Dashboard page you're on
    /// Responds with ::Dashboard and subsequent ::ElementUpdate's
    /// Note: u16::MAX is used for non-dashboard type pages
    SetDashboard(u16),
}

/// Igloo -> WASM
#[derive(Clone, BorshSerialize, BorshDeserialize, From)]
pub enum ServerMessage {
    Dashboard(Box<Dashboard>),
    ElementUpdate(ElementUpdate),
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct ElementUpdate {
    pub elid: u32,
    pub value: Component,
}
