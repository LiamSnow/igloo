use clap::command;
use clap_derive::{Args, Parser, Subcommand, ValueEnum};
use clap::Parser;
use serde::Serialize;

use crate::command::Color;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommands,
}

impl Cli {
    pub fn parse(cmd_str: &str) -> Result<Self, clap::error::Error> {
        let cmd_str = "igloo ".to_string() + cmd_str;
        let res = Self::try_parse_from(cmd_str.split_whitespace())?;
        Ok(res)
    }
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    /// Control lights
    #[command(alias = "lights")]
    Light(LightArgs),
    /// Control switches
    #[command(alias = "switches")]
    Switch(SwitchArgs),
    /// UI Interface
    UI,
    /// List various items
    #[command(alias = "ls")]
    List(ListArgs),
    /// Describe various items
    // #[command(alias = "dsc")]
    // Describe(DescribeArgs),
    /// View logs
    Logs(LogsArgs),
    /// Control automations
    #[command(alias = "atm")]
    Automation(AutomationArgs),
    /// Reload the system
    Reload,
    /// Display version information
    Version,
}

#[derive(Args, Debug)]
pub struct LightArgs {
    /// Target light
    pub target: String,
    #[command(subcommand)]
    pub action: LightAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum LightAction {
    /// Turn the light on
    On,
    /// Turn the light off
    Off,
    /// Set the light color
    #[command(alias = "rgb")]
    Color(Color),
    /// Set the light temperature
    #[command(alias = "temp")]
    Temperature {
        temp: u32,
    },
    /// Set the light brightness
    #[command(alias = "bri")]
    Brightness {
        brightness: u8,
    },
}

#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// Target switch
    pub target: String,
    /// Turn the switch on or off
    #[arg(value_enum)]
    pub state: SwitchState,
}

#[derive(ValueEnum, Clone, Debug, Serialize)]
pub enum SwitchState {
    On,
    Off,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self::Off
    }
}

impl From<SwitchState> for bool {
    fn from(value: SwitchState) -> Self {
        match value {
            SwitchState::On => true,
            SwitchState::Off => false
        }
    }
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[command(subcommand)]
    pub item: ListItems,
}

#[derive(Subcommand, Debug)]
pub enum ListItems {
    /// List users
    #[command(alias = "usrs")]
    Users,
    /// List user groups
    #[command(alias = "ugs")]
    UserGroups,
    /// List providers
    #[command(alias = "pvds")]
    Providers,
    /// List automations
    #[command(alias = "atms")]
    Automations,
    /// List zones
    #[command(alias = "zns")]
    Zones,
    /// List devices in zone
    #[command(alias = "devs")]
    Devices { zone: String },
    /// List subdevices in device
    #[command(alias = "subdevs")]
    Subdevices { dev: String },
}

#[derive(Args, Debug)]
pub struct DescribeArgs {
    #[command(subcommand)]
    pub item: DescribeItems,
}

#[derive(Subcommand, Debug)]
pub enum DescribeItems {
    // /// Describe a user
    // #[command(alias = "usr")]
    // User { user: String },
    // /// Describe a user group
    // #[command(alias = "ug")]
    // UserGroup { user_group: String },
    // /// Describe a provider
    // #[command(alias = "pvd")]
    // Provider { provider: String },
    // /// Describe a zone
    // #[command(alias = "zn")]
    // Zone { zone: String },
    /// Describe a device
    // #[command(alias = "dev")]
    // Device { dev: String },
    /// Describe an automation
    #[command(alias = "atm")]
    Automation { automation: String },
}

#[derive(Args, Debug)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub log_type: LogType,
}

#[derive(Subcommand, Debug)]
pub enum LogType {
    /// View system logs
    System,
    /// View user logs
    #[command(alias = "usr")]
    User { user: String },
    /// View device logs
    #[command(alias = "dev")]
    Device { dev: String },
    /// View automation logs
    #[command(alias = "atm")]
    Automation { automation: String },
}

#[derive(Args, Debug)]
pub struct AutomationArgs {
    /// Target automation
    pub automation: String,
    #[command(subcommand)]
    pub action: AutomationAction,
}

#[derive(Subcommand, Debug)]
pub enum AutomationAction {
    /// Trigger the automation
    Trigger,
    /// Get or set the automation value
    Value(AutomationValue),
}

#[derive(Args, Debug)]
pub struct AutomationValue {
    #[command(subcommand)]
    pub action: AutomationValueAction,
}

#[derive(Subcommand, Debug)]
pub enum AutomationValueAction {
    /// Set the automation value
    Set { value: String },
    /// Get the automation value
    Get,
}
