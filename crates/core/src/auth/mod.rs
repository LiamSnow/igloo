use std::time::Duration;

pub mod file;
pub mod model;
pub mod modify;

pub const CONFIG_VERSION: u32 = 1;
/// 10 days
pub const SESSION_DURATION: Duration = Duration::from_secs(10 * 24 * 3600);

pub use model::*;
