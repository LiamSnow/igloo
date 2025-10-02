#[cfg(feature = "igloo-server")]
include!(concat!(env!("OUT_DIR"), "/server.rs"));

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[cfg(feature = "igloo-server")]
pub mod avg;

#[cfg(test)]
mod tests;

pub mod helpers;
pub use helpers::*;
