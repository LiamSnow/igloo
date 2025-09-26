pub mod avg;
pub mod entity;
pub mod interface;
#[cfg(test)]
mod test;

pub use avg::*;
pub use entity::*;
pub use interface::*;

include!(concat!(env!("OUT_DIR"), "/components.rs"));
