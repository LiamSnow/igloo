include!(concat!(env!("OUT_DIR"), "/out.rs"));

// TODO rename to somrthing else
include!(concat!(env!("OUT_DIR"), "/server.rs"));

pub mod id;
pub mod query;
pub mod types;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "floe")]
pub mod floe;

#[cfg(feature = "penguin")]
pub mod penguin;
