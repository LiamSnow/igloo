use std::error::Error;

use config::IglooConfig;
use state::IglooState;

pub mod cli;
pub mod config;
pub mod scripts;
pub mod state;
pub mod providers;
pub mod selector;
pub mod elements;
pub mod auth;
pub mod subdevice;
pub mod device;
pub mod api;

pub const VERSION: f32 = 0.1;
pub const CONFIG_VERSION: f32 = 0.1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = IglooConfig::from_file("./config.ron").unwrap();
    if cfg.version != CONFIG_VERSION {
        panic!(
            "Wrong config version. Got {}, expected {}.",
            cfg.version, CONFIG_VERSION
        );
    }

    let state = IglooState::init(cfg).await?;

    api::init(state).await?;

    Ok(())
}

