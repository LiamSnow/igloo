pub mod avg;
pub mod codec;
pub mod floe;

#[cfg(test)]
mod tests;

pub mod components {
    include!(concat!(env!("OUT_DIR"), "/components.rs"));
}

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}

pub use avg::*;
pub use codec::*;
pub use components::*;
pub use protocol::*;
