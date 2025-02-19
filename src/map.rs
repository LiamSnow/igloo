use std::{collections::HashMap, error::Error, sync::Arc};

use tokio::{sync::RwLock, task::JoinSet};

use crate::{config::IglooZones, device::device::IglooDeviceLock, providers::IglooDevice};

pub type DeviceMap = Arc<HashMap<String, ZoneMap>>;
pub type ZoneMap = HashMap<String, IglooDeviceLock>;

pub fn make(zones: IglooZones) -> Result<DeviceMap, Box<dyn Error>> {
    let mut device_table = HashMap::new();
    for (zone_name, devices) in zones {
        let mut zone_table = HashMap::new();
        for (device_name, device_config) in devices {
            zone_table.insert(device_name, Arc::new(RwLock::new(IglooDevice::make(device_config)?)));
        }
        device_table.insert(zone_name, zone_table);
    }
    Ok(Arc::new(device_table))
}

pub async fn connect_all(table: DeviceMap) {
    let mut set = JoinSet::new();
    for (_, zone) in &*table {
        for (_, dev_lock) in zone {
            set.spawn(IglooDevice::connect(dev_lock.clone()));
        }
    }
    set.join_all().await;
}
