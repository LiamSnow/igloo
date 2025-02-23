use std::{collections::HashMap, error::Error, sync::Arc};

use serde::Serialize;
use thiserror::Error;
use tokio::{sync::RwLock, task::JoinSet};

use crate::{config::{IglooConfig, UIElementConfig, ZonesConfig}, device::{command::LightState, device::IglooDeviceLock}, providers::IglooDevice};

pub struct IglooStack {
    pub map: ZonesMap,
    pub ui: HashMap<String, Vec<UIElement>>
}
pub type ZonesMap = Arc<HashMap<String, ZoneMap>>;
pub type ZoneMap = Arc<HashMap<String, IglooDeviceLock>>;

#[derive(Serialize)]
pub struct UIElement {
    pub state: UIElementState,
}

#[derive(Serialize)]
pub enum UIElementState {
    Light(LightState),
    Switch,
    Button,
    Selector
}

impl IglooStack {
    pub async fn init(config: IglooConfig) -> Result<Self, Box<dyn Error>> {
        let map = Self::make_map(config.zones)?;
        Self::connect_map(map.clone()).await;
        let ui = Self::make_ui(map.clone(), config.ui)?;

        Ok(IglooStack { map, ui })
    }

    fn make_map(zones: ZonesConfig) -> Result<ZonesMap, Box<dyn Error>> {
        let mut map = HashMap::new();
        for (zone_name, devices) in zones {
            let mut zone_map = HashMap::new();
            for (device_name, device_config) in devices {
                zone_map.insert(device_name, Arc::new(RwLock::new(IglooDevice::make(device_config)?)));
            }
            map.insert(zone_name, Arc::new(zone_map));
        }
        Ok(Arc::new(map))
    }

    async fn connect_map(map: ZonesMap) {
        let mut set = JoinSet::new();
        for (_, zone) in &*map {
            for (_, dev_lock) in &**zone {
                set.spawn(IglooDevice::connect(dev_lock.clone()));
            }
        }
        set.join_all().await;
    }

    fn make_ui(map: ZonesMap, cfg: HashMap<String, Vec<UIElementConfig>>) -> Result<HashMap<String, Vec<UIElement>>, Box<dyn Error>> {

    }
}

pub enum Selector {
    All(ZonesMap),
    Zone(ZoneMap),
    Device(IglooDeviceLock),
    Subdevice(IglooDeviceLock, String)
}

#[derive(Error, Debug, Serialize)]
pub enum SelectorError {
    #[error("scope selector must be `all`, ZONE, ZONE.DEVICE, or ZONE.DEVICE.SUBDEVICE")]
    BadSelector,
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
    #[error("unknown device `{0}.{1}`")]
    UnknownDevice(String, String),
    #[error("expected all, but got something else")]
    ExpectedAll,
    #[error("expected zone, but got something else")]
    ExpectedZone,
    #[error("expected device, but got something else")]
    ExpectedDevice,
    #[error("expected subdevice, but got something else")]
    ExpectedSubdevice,
}

impl Selector {
    pub fn from_str(map: ZonesMap, selector: &str) -> Result<Self, SelectorError> {
        if selector == "all" {
            return Ok(Self::All(map))
        }

        let parts: Vec<String> = selector.split(".").map(|s| s.to_string()).collect();
        if parts.len() < 1 || parts.len() > 3 {
            return Err(SelectorError::BadSelector);
        }

        let zone_name = parts.get(0).unwrap();
        let zone = map.get(zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;

        if let Some(dev_name) = parts.get(1) {
            let dev = zone.get(dev_name).ok_or(SelectorError::UnknownDevice(zone_name.to_string(), dev_name.to_string()))?;

            if let Some(subdev_name) = parts.get(2) {
                Ok(Self::Subdevice(dev.clone(), subdev_name.to_string()))
            }
            else {
                Ok(Self::Device(dev.clone()))
            }
        }
        else {
            Ok(Self::Zone(zone.clone()))
        }
    }

    pub fn get_all(self) -> Result<ZonesMap, SelectorError> {
        match self {
            Selector::All(o) => Ok(o),
            _ => Err(SelectorError::ExpectedAll)
        }
    }

    pub fn get_zone(self) -> Result<ZoneMap, SelectorError> {
        match self {
            Selector::Zone(o) => Ok(o),
            _ => Err(SelectorError::ExpectedZone)
        }
    }

    pub fn get_device(self) -> Result<IglooDeviceLock, SelectorError> {
        match self {
            Selector::Device(o) => Ok(o),
            _ => Err(SelectorError::ExpectedDevice)
        }
    }

    pub fn get_subdevice(self) -> Result<(IglooDeviceLock, String), SelectorError> {
        match self {
            Selector::Subdevice(d, s) => Ok((d, s)),
            _ => Err(SelectorError::ExpectedSubdevice)
        }
    }
}

