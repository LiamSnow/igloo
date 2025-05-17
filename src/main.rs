use std::{env, error::Error};

use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use config::IglooConfig;
use state::IglooState;

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod device;
pub mod elements;
pub mod entity;
pub mod scripts;
pub mod state;

pub const VERSION: f32 = 0.1;
pub const CONFIG_VERSION: f32 = 0.1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // args
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        return handle_args(args);
    }

    // config
    let cfg = IglooConfig::from_file("./config.ron");
    if cfg.version != CONFIG_VERSION {
        panic!(
            "Wrong config version. Got {}, expected {}.",
            cfg.version, CONFIG_VERSION
        );
    }

    // logging
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_timer(fmt::time::time())
                .with_ansi(true)
                .with_target(false)
                .with_writer(std::io::stdout),
        )
        .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    // setup
    let state = IglooState::init(cfg).await?;

    // run
    api::run(state).await?;

    Ok(())
}

fn handle_args(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    match args.get(1).unwrap().as_str() {
        "hash" => {
            let passwd = args.get(2).ok_or("Please provide password.")?;
            let hashed = bcrypt::hash(passwd, bcrypt::DEFAULT_COST)?;
            println!("{hashed}");
        }
        _ => return Err("Unknown argument".into()),
    }
    Ok(())
}
