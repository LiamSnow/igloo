use std::{env, error::Error};

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
pub mod entity;
pub mod device;
pub mod api;

pub const VERSION: f32 = 0.1;
pub const CONFIG_VERSION: f32 = 0.1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(arg) if arg == "hash" => {
            let passwd = args.get(2).ok_or("Please provide password.")?;
            let hashed = bcrypt::hash(passwd, bcrypt::DEFAULT_COST)?;
            println!("{hashed}");
            return Ok(())
        },
        _ => {},
    }

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

