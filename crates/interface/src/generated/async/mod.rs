pub mod shared;
pub use shared::igloo::lib::*;

#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "core")]
pub use core::igloo::lib::*;

#[cfg(all(feature = "extension", feature = "single-process"))]
pub mod extension_sp;
#[cfg(all(feature = "extension", feature = "single-process"))]
pub use extension_sp::igloo::lib::*;

#[cfg(all(feature = "extension", not(feature = "single-process")))]
pub mod extension_mp;
#[cfg(all(feature = "extension", not(feature = "single-process")))]
pub use extension_mp::igloo::lib::*;
