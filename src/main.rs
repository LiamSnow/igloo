use std::{collections::HashMap, sync::Arc};

use config::{IglooConfig, IglooZones};
use device::{IglooDeviceArc, IglooDevice};
use command::{CTLightCommand, IglooCommand};
use tokio::{sync::RwLock, task::JoinSet};

pub mod config;
pub mod command;
pub mod device;
pub mod parser;

pub type DeviceMap = Arc<HashMap<String, ZoneMap>>;
pub type ZoneMap = HashMap<String, IglooDeviceArc>;

fn make_device_table(zones: IglooZones) -> DeviceMap {
    let mut device_table = HashMap::new();
    for (zone_name, devices) in zones {
        let mut zone_table = HashMap::new();
        for (device_name, device_config) in devices {
            zone_table.insert(device_name, Arc::new(RwLock::new(device_config.into())));
        }
        device_table.insert(zone_name, zone_table);
    }
    Arc::new(device_table)
}

async fn connect_all_devices(dev_map: DeviceMap) {
    let mut set = JoinSet::new();
    for (_, devs) in &*dev_map {
        for (_, dev_arc) in devs {
            set.spawn(IglooDevice::connect_arc(dev_arc.clone()));
        }
    }
    set.join_all().await;
}

async fn all_command(dev_map: DeviceMap, cmd: IglooCommand) {
    let mut set = JoinSet::new();
    for (_, devs) in &*dev_map {
        for (_, dev_arc) in devs {
            set.spawn(IglooDevice::command_arc(dev_arc.clone(), cmd.clone()));
        }
    }
    set.join_all().await;
}

pub const VERSION: f32 = 0.1;

#[tokio::main]
async fn main() {
    let cfg = IglooConfig::from_file("./config.ron").unwrap();
    if cfg.version != VERSION {
        panic!("Wrong config version. Got {}, expected {}.", cfg.version, VERSION);
    }

    let dev_map = make_device_table(cfg.zones);
    connect_all_devices(dev_map.clone()).await;

    let cmds = vec![
        "bar.a light off",
    ];

    let cmd = IglooCommand::CTLightCommand(CTLightCommand {
        state: Some(true),
        brightness: Some(1.0),
        temp: Some(2000.)
    });
    all_command(dev_map.clone(), cmd).await;


}
