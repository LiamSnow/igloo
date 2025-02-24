use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::{command::{RackSubdeviceCommand, SubdeviceCommand}, map::DeviceCmdChannelMap};

pub enum Selection {
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

impl Selection {
    pub fn new(map: &DeviceCmdChannelMap, selection_str: &str) -> Result<Self, SelectorError> {
        match SelectionString::new(selection_str)? {
            SelectionString::All => {
                let mut v = Vec::new();
                for (_, zone) in map {
                    for (_, dev) in zone {
                        v.push(dev.clone());
                    }
                }
                return Ok(Self::Multiple(v))
            },
            SelectionString::Zone(zone_name) => {
                let zone = map.get(zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let mut v = Vec::new();
                for (_, dev) in zone {
                    v.push(dev.clone());
                }
                Ok(Self::Multiple(v))
            },
            SelectionString::Device(zone_name, dev_name) => {
                let zone = map.get(zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let dev = zone.get(dev_name).ok_or(SelectorError::UnknownDevice(zone_name.to_string(), dev_name.to_string()))?;
                Ok(Self::Device(dev.clone()))
            },
            SelectionString::Subdevice(zone_name, subdev_name, dev_name) => {
                let zone = map.get(zone_name).ok_or(SelectorError::UnknownZone(zone_name.to_string()))?;
                let dev = zone.get(dev_name).ok_or(SelectorError::UnknownDevice(zone_name.to_string(), dev_name.to_string()))?;
                Ok(Self::Subdevice(dev.clone(), subdev_name.to_string()))
            },
        }
    }

    pub fn get_multiple(self) -> Result<Vec<Sender<RackSubdeviceCommand>>, SelectorError> {
        match self {
            Selection::Multiple(o) => Ok(o),
            _ => Err(SelectorError::ExpectedZone)
        }
    }

    pub fn get_device(self) -> Result<Sender<RackSubdeviceCommand>, SelectorError> {
        match self {
            Selection::Device(o) => Ok(o),
            _ => Err(SelectorError::ExpectedDevice)
        }
    }

    pub fn get_subdevice(self) -> Result<(Sender<RackSubdeviceCommand>, String), SelectorError> {
        match self {
            Selection::Subdevice(d, s) => Ok((d, s)),
            _ => Err(SelectorError::ExpectedSubdevice)
        }
    }

    pub fn execute(&mut self, cmd: SubdeviceCommand) -> Result<(), SelectorError> {
        match self {
            Selection::Multiple(devs) => {
                for dev in devs {
                    dev.try_send(RackSubdeviceCommand {
                        cmd: cmd.clone(),
                        subdev_name: None,
                    }).map_err(|_| SelectorError::DeviceChannelFull)?;
                }
			},
            Selection::Device(dev) => {
                dev.try_send(RackSubdeviceCommand {
                    cmd,
                    subdev_name: None,
                }).map_err(|_| SelectorError::DeviceChannelFull)?;
			},
            Selection::Subdevice(dev, subdev_name) => {
                dev.try_send(RackSubdeviceCommand {
                    cmd,
                    subdev_name: Some(subdev_name.to_string()),
                }).map_err(|_| SelectorError::DeviceChannelFull)?;
			},
        }

        Ok(())
    }
}

pub enum SelectionString<'a> {
    All,
    Zone(&'a str),
    Device(&'a str, &'a str),
    Subdevice(&'a str, &'a str, &'a str)
}

impl<'a> SelectionString<'a> {
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
}

