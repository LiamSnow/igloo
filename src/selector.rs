use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::{command::{RackSubdeviceCommand, SubdeviceCommand}, map::DeviceCommandChannelMap};

pub enum DevCommandChannelRef {
    Multiple(Vec<Sender<RackSubdeviceCommand>>),
    Device(Sender<RackSubdeviceCommand>),
    Subdevice(Sender<RackSubdeviceCommand>, String),
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
    #[error("device channel is full")]
    DeviceChannelFull,
}

/// Stores a copy of device command channels given a selection
impl DevCommandChannelRef {
    pub fn from_str(map: &DeviceCommandChannelMap, selection_str: &str) -> Result<Self, SelectorError> {
        Self::new(map, &Selection::new(selection_str)?)
    }

    pub fn new(map: &DeviceCommandChannelMap, selection: &Selection) -> Result<Self, SelectorError> {
        match selection {
            Selection::All => {
                let mut v = Vec::new();
                for (_, zone) in map {
                    for (_, dev) in zone {
                        v.push(dev.clone());
                    }
                }
                return Ok(Self::Multiple(v))
            },
            Selection::Zone(zone_name) => {
                let zone = map.get(*zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let mut v = Vec::new();
                for (_, dev) in zone {
                    v.push(dev.clone());
                }
                Ok(Self::Multiple(v))
            },
            Selection::Device(zone_name, dev_name) => {
                let zone = map.get(*zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let dev = zone.get(*dev_name).ok_or(SelectorError::UnknownDevice(zone_name.to_string(), dev_name.to_string()))?;
                Ok(Self::Device(dev.clone()))
            },
            Selection::Subdevice(zone_name, subdev_name, dev_name) => {
                let zone = map.get(*zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let dev = zone.get(*dev_name).ok_or(SelectorError::UnknownDevice(zone_name.to_string(), dev_name.to_string()))?;
                Ok(Self::Subdevice(dev.clone(), subdev_name.to_string()))
            },
        }
    }

    pub fn get_multiple(self) -> Result<Vec<Sender<RackSubdeviceCommand>>, SelectorError> {
        match self {
            DevCommandChannelRef::Multiple(o) => Ok(o),
            _ => Err(SelectorError::ExpectedZone)
        }
    }

    pub fn get_device(self) -> Result<Sender<RackSubdeviceCommand>, SelectorError> {
        match self {
            DevCommandChannelRef::Device(o) => Ok(o),
            _ => Err(SelectorError::ExpectedDevice)
        }
    }

    pub fn get_subdevice(self) -> Result<(Sender<RackSubdeviceCommand>, String), SelectorError> {
        match self {
            DevCommandChannelRef::Subdevice(d, s) => Ok((d, s)),
            _ => Err(SelectorError::ExpectedSubdevice)
        }
    }

    pub fn execute(&mut self, cmd: SubdeviceCommand) -> Result<(), SelectorError> {
        match self {
            DevCommandChannelRef::Multiple(devs) => {
                for dev in devs {
                    dev.try_send(RackSubdeviceCommand {
                        cmd: cmd.clone(),
                        subdev_name: None,
                    }).map_err(|_| SelectorError::DeviceChannelFull)?;
                }
			},
            DevCommandChannelRef::Device(dev) => {
                dev.try_send(RackSubdeviceCommand {
                    cmd,
                    subdev_name: None,
                }).map_err(|_| SelectorError::DeviceChannelFull)?;
			},
            DevCommandChannelRef::Subdevice(dev, subdev_name) => {
                dev.try_send(RackSubdeviceCommand {
                    cmd,
                    subdev_name: Some(subdev_name.to_string()),
                }).map_err(|_| SelectorError::DeviceChannelFull)?;
			},
        }

        Ok(())
    }
}

pub enum Selection<'a> {
    All,
    Zone(&'a str),
    Device(&'a str, &'a str),
    Subdevice(&'a str, &'a str, &'a str)
}

impl<'a> Selection<'a> {
    pub fn new(selection_str: &'a str) -> Result<Self, SelectorError> {
        if selection_str == "all" {
            return Ok(Self::All)
        }

        let parts: Vec<&str> = selection_str.split(".").collect();
        if parts.len() < 1 || parts.len() > 3 {
            return Err(SelectorError::BadSelector);
        }

        let zone_name = parts.get(0).unwrap();

        if let Some(dev_name) = parts.get(1) {
            if let Some(subdev_name) = parts.get(2) {
                Ok(Self::Subdevice(zone_name, dev_name, subdev_name))
            }
            else {
                Ok(Self::Device(zone_name, dev_name))
            }
        }
        else {
            Ok(Self::Zone(zone_name))
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Selection::All => 3,
            Selection::Zone(..) => 2,
            Selection::Device(..) => 1,
            Selection::Subdevice(..) => 0,
        }
    }

    pub fn get_zone(&self) -> Option<&str> {
        match self {
            Selection::All => None,
            Selection::Zone(z) => Some(z),
            Selection::Device(z, _) => Some(z),
            Selection::Subdevice(z, _, _) => Some(z),
        }
    }

    pub fn get_dev(&self) -> Option<&str> {
        match self {
            Selection::All => None,
            Selection::Zone(_) => None,
            Selection::Device(_, d) => Some(d),
            Selection::Subdevice(_, d, _) => Some(d),
        }
    }

    pub fn get_subdev(&self) -> Option<&str> {
        match self {
            Selection::All => None,
            Selection::Zone(_) => None,
            Selection::Device(_, _) => None,
            Selection::Subdevice(_, _, s) => Some(s),
        }
    }

    pub fn collides(&self, other: &Selection) -> bool {
        if other.rank() > self.rank() {
            return other.collides(self)
        }
        match self {
            Selection::All => true,
            Selection::Zone(zone) => {
                *zone == other.get_zone().unwrap()
            },
            Selection::Device(zone, dev) => {
                *zone == other.get_zone().unwrap() &&
                *dev == other.get_dev().unwrap()
            },
            Selection::Subdevice(zone, dev, sub) => {
                *zone == other.get_zone().unwrap() &&
                *dev == other.get_dev().unwrap() &&
                *sub == other.get_subdev().unwrap()
            },
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum OwnedSelection {
    All,
    Zone(String),
    Device(String, String),
    Subdevice(String, String, String)
}

impl From<Selection<'_>> for OwnedSelection {
    fn from(value: Selection) -> Self {
        match value {
            Selection::All => Self::All,
            Selection::Zone(z) => Self::Zone(z.to_string()),
            Selection::Device(z, d) => Self::Device(z.to_string(), d.to_string()),
            Selection::Subdevice(z, d, s) => Self::Subdevice(z.to_string(), d.to_string(), s.to_string())
        }
    }
}

impl<'a> From<&'a OwnedSelection> for Selection<'a> {
    fn from(value: &'a OwnedSelection) -> Self {
        match value {
            OwnedSelection::All => Self::All,
            OwnedSelection::Zone(z) => Self::Zone(z),
            OwnedSelection::Device(z, d) => Self::Device(z, d),
            OwnedSelection::Subdevice(z, d, s) => Self::Subdevice(z, d, s),
        }
    }
}

impl OwnedSelection {
    pub fn collides(&self, other: &Selection) -> bool {
        other.collides(&self.into())
    }
}
