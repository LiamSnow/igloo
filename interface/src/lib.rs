include!(concat!(env!("OUT_DIR"), "/out.rs"));

pub mod id;
pub mod query;
pub mod types;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "ipc")]
pub mod ipc;

#[cfg(feature = "penguin")]
pub mod penguin;
