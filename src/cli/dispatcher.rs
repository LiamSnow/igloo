use std::error::Error;

use crate::{device::{command::DeviceCommand, scoped::ScopedDeviceCommand}, map::DeviceMap};

use super::model::{Cli, Commands, ListItems};

impl Cli {
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
                    ListItems::Target { target: _ } => todo!(),
                }
            },
            Commands::Describe(_) => todo!(),
            Commands::Logs(_) => todo!(),
            Commands::Automation(_) => todo!(),
            Commands::Reload => todo!(),
            Commands::Version => todo!(),
            Commands::Top => todo!(),
        }

        Ok(())
    }
}
