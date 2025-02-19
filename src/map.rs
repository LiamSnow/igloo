use std::{collections::HashMap, error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{config::IglooZones, device::device::{IglooDevice, IglooDeviceArc}};

pub type DeviceMap = Arc<HashMap<String, ZoneMap>>;
pub type ZoneMap = HashMap<String, IglooDeviceArc>;

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

