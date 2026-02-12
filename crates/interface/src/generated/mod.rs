#[cfg(feature = "async")]
pub mod r#async;
#[cfg(feature = "async")]
pub use r#async::*;

#[cfg(not(feature = "async"))]
pub mod sync;
#[cfg(not(feature = "async"))]
pub use sync::*;
