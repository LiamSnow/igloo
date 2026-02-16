use crate::core::IglooRequest;
use clap::Parser;
use std::{path::PathBuf, sync::OnceLock};

mod core;
mod ext;
mod history;
mod query;
mod tree;
mod web;

#[derive(Parser, Debug)]
#[command(name = "igloo")]
struct Args {
    /// Hostname or IP address to bind to
    #[arg(short, long, env = "IGLOO_ADDRESS", default_value = "127.0.0.1")]
    address: String,

    /// Port number (1-65535)
    #[arg(short, long, env = "IGLOO_PORT", default_value_t = 4299)]
    port: u16,

    /// Path to the `data` dir
    #[arg(long, env = "IGLOO_DATA", default_value = "./data")]
    data_dir: String,

    /// Path to the `packages` dir
    #[arg(long, env = "IGLOO_PACKAGES", default_value = "./packages")]
    packages_dir: String,

    /// Path to the `www` dir (frontend)
    #[arg(long, env = "IGLOO_WWW", default_value = "./www")]
    www_dir: String,
}

pub static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static PACKAGES_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static WWW_DIR: OnceLock<PathBuf> = OnceLock::new();

#[tokio::main]
async fn main() {
    let args = Args::parse();

    DATA_DIR.set(PathBuf::from(args.data_dir)).unwrap();
    PACKAGES_DIR.set(PathBuf::from(args.packages_dir)).unwrap();
    WWW_DIR.set(PathBuf::from(args.www_dir)).unwrap();

    let (handle, req_tx) = match core::spawn().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    if let Err(e) = web::run(req_tx.clone(), args.address, args.port).await {
        eprintln!("Error running web: {e}");
    }

    tokio::signal::ctrl_c().await.unwrap();
    println!("SHUTTING DOWN");
    req_tx.send(IglooRequest::Shutdown).unwrap();
    handle.join().unwrap();
}
