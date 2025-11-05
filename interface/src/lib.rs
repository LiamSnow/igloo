#[cfg(feature = "server")]
include!(concat!(env!("OUT_DIR"), "/server.rs"));

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub use server::*;

#[cfg(feature = "floe")]
pub mod floe;

#[cfg(feature = "penguin")]
pub mod penguin;

pub mod types;
