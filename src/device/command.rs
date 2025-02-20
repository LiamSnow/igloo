use std::error::Error;

use clap_derive::Args;
use tokio::task::JoinSet;

use crate::{cli::model::LightAction, map::DeviceMap, providers::IglooDevice};

#[derive(Debug, Clone)]
pub enum DeviceCommand {
    Light(LightAction),
}

#[derive(Debug, Default, Clone, Args)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub struct ScopedDeviceCommand {
    pub zone: Option<String>,
    pub dev: Option<String>,
    pub subdev: Option<String>,
    pub cmd: DeviceCommand
}

impl ScopedDeviceCommand {
    pub fn from_str(target: &str, cmd: DeviceCommand) -> Result<Self, Box<dyn Error>> {
        if target == "all" {
            return Ok(Self::all(cmd));
        }

        let parts: Vec<String> = target.split(".").map(|s| s.to_string()).collect();
        if parts.len() < 1 || parts.len() > 3 {
            return Err("command target has wrong number of parts".into());
        }

        Ok(Self {
            cmd,
            zone: parts.get(0).cloned(),
            dev: parts.get(1).cloned(),
            subdev: parts.get(2).cloned()
        })
    }

    pub fn all(cmd: DeviceCommand) -> Self {
        Self {
            cmd,
            zone: None,
            dev: None,
            subdev: None
        }
    }

    pub async fn execute(self, table: DeviceMap) -> Result<(), Box<dyn Error>> {
        if let Some(zone) = self.zone {
            let zone = table.get(&zone).ok_or("could not find zone")?;

            if let Some(device) = &self.dev {
                let device = zone.get(device).ok_or("could not find device")?;

                //subdevice
                if let Some(subdevice) = self.subdev {
                    IglooDevice::execute_lock(device.clone(), self.cmd, subdevice).await;
                }

                //device
                else {
                    IglooDevice::execute_global_lock(device.clone(), self.cmd).await;
                }
            }

            //zone
            else {
                let mut set = JoinSet::new();
                for (_, dev_lock) in zone {
                    set.spawn(IglooDevice::execute_global_lock(dev_lock.clone(), self.cmd.clone()));
                }
                set.join_all().await;
            }
        }

        //all
        else {
            let mut set = JoinSet::new();
            for (_, zone) in &*table {
                for (_, dev_lock) in zone {
                    set.spawn(IglooDevice::execute_global_lock(dev_lock.clone(), self.cmd.clone()));
                }
            }
            set.join_all().await;
        }

        Ok(())
    }
}

