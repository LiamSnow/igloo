#[cfg(feature = "server")]
include!(concat!(env!("OUT_DIR"), "/server.rs"));

include!(concat!(env!("OUT_DIR"), "/out.rs"));

#[cfg(feature = "server")]
pub mod avg;

#[cfg(test)]
mod tests;

#[cfg(feature = "floe")]
pub mod helpers;
#[cfg(feature = "floe")]
pub use helpers::*;
