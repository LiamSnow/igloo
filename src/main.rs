use std::error::Error;

use cli::model::Cli;
use config::IglooConfig;
use map::connect_all;

pub mod config;
pub mod device;
pub mod cli;
pub mod map;
pub mod providers;

pub const VERSION: f32 = 0.1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = IglooConfig::from_file("./config.ron").unwrap();
    if cfg.version != VERSION {
        panic!("Wrong config version. Got {}, expected {}.", cfg.version, VERSION);
    }

    let table = map::make(cfg.zones)?;
    connect_all(table.clone()).await;
    println!("all connected!");

    loop {
        let mut cmd_str = String::new();
        println!("Command: ");
        std::io::stdin().read_line(&mut cmd_str).unwrap();
        if &cmd_str == "exit" {
            break;
        }

        let cmd = Cli::parse(&cmd_str)?;
        cmd.dispatch(table.clone()).await?;
    }

    Ok(())
}
