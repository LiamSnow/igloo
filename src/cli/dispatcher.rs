use std::error::Error;

use crate::{device::command::{DeviceCommand, ScopedDeviceCommand}, map::DeviceMap, VERSION};

use super::model::{Cli, Commands, DescribeItems, ListItems, LogType};

impl Cli {
    //FIXME return json instead of printing
    pub async fn dispatch(self, table: DeviceMap) -> Result<(), Box<dyn Error>> {
        match self.command {
            Commands::Light(args) => {
                let cmd = ScopedDeviceCommand::from_str(
                    &args.target,
                    DeviceCommand::Light(args.action)
                )?;
                cmd.execute(table).await?;
            },
            Commands::Switch(_) => todo!(),
            Commands::List(args) => {
                match args.item {
                    ListItems::Users => todo!(),
                    ListItems::UserGroups => todo!(),
                    ListItems::Providers => todo!(),
                    ListItems::Automations => todo!(),
                    ListItems::Zones => {
                        for (zone_name, _) in &*table {
                            println!("{zone_name}");
                        }
                    },
                    ListItems::Devices { zone } => {
                        let zone = table.get(&zone).ok_or("could not find zone")?;
                        for (dev_name, _) in zone {
                            println!("{dev_name}");
                        }
                    },
                    ListItems::Subdevices { dev } => {
                        //TODO func
                        let (zone_name, dev_name) =  dev.split_once(".").ok_or("please provide ZONE.DEVICE")?;
                        let zone = table.get(zone_name).ok_or("could not find zone")?;
                        let dev_lock = zone.get(dev_name).ok_or("could not find device")?;

                        let dev = dev_lock.read().await;
                        for subdev in dev.list_subdevs() {
                            println!("{subdev}");
                        }
                    },
                }
            },
            Commands::Describe(args) => {
                match args.item {
                    DescribeItems::Device { dev } => {
                        //TODO func
                        let (zone_name, dev_name) =  dev.split_once(".").ok_or("please provide ZONE.DEVICE")?;
                        let zone = table.get(zone_name).ok_or("could not find zone")?;
                        let dev_lock = zone.get(dev_name).ok_or("could not find device")?;

                        let dev = dev_lock.read().await;
                        println!("{}", dev.describe());
                    },
                    DescribeItems::Automation { automation: _ } => {
                        todo!()
                    },
                }
            },
            Commands::Logs(args) => {
                match args.log_type {
                    LogType::System => todo!(),
                    LogType::User { user: _ } => todo!(),
                    LogType::Device { dev } => {
                        let (zone_name, dev_name) =  dev.split_once(".").ok_or("please provide ZONE.DEVICE")?;
                        let zone = table.get(zone_name).ok_or("could not find zone")?;
                        let dev_lock = zone.get(dev_name).ok_or("could not find device")?;

                        let mut dev = dev_lock.write().await;
                        dev.subscribe_logs().await;
                    },
                    LogType::Automation { automation: _ } => todo!(),
                }
            },
            Commands::Automation(_) => todo!(),
            Commands::Reload => todo!(),
            Commands::Version => println!("{}", VERSION),
        }

        Ok(())
    }
}
