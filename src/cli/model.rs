use clap::command;
use clap::Parser;
use clap_derive::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::command::SubdeviceType;

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
    UI(UIArgs),
    /// List various items
    #[command(alias = "ls")]
    List(ListArgs),
    /// Describe various items
    // #[command(alias = "dsc")]
    // Describe(DescribeArgs),
    /// View logs
    Logs(LogsArgs),
    /// Control scripts
    #[command(alias = "scr")]
    Script(ScriptArgs),
    /// Reload the system
    Reload,
    /// Display version information
    Version,
}

impl CliCommands {
    pub fn get_selection(&self) -> Option<&str> {
        Some(match self {
            Self::Light(args) => &args.target,
            Self::Switch(args) => &args.target,
            Self::List(args) => return args.item.get_selection(),
            _ => return None,
        })
    }

    pub fn get_subdev_type(&self) -> Option<SubdeviceType> {
        Some(match self {
            Self::Light(..) => SubdeviceType::Light,
            Self::Switch(..) => SubdeviceType::Switch,
            _ => return None
        })
    }
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
    /// Set the light color using an hue value
    #[command(alias = "hue")]
    Color { hue: Option<u16> },
    /// Set the light temperature
    #[command(alias = "temp")]
    Temperature { temp: Option<u32> },
    /// Set the light brightness
    #[command(alias = "bri")]
    Brightness { brightness: u8 },
}

#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// Target switch
    pub target: String,
    /// Turn the switch on or off
    #[arg(value_enum)]
    pub action: SwitchState,
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

impl From<bool> for SwitchState {
    fn from(value: bool) -> Self {
        match value {
            true => SwitchState::On,
            false => SwitchState::Off,
        }
    }
}

impl From<SwitchState> for bool {
    fn from(value: SwitchState) -> Self {
        match value {
            SwitchState::On => true,
            SwitchState::Off => false,
        }
    }
}

impl From<&SwitchState> for bool {
    fn from(value: &SwitchState) -> Self {
        match value {
            SwitchState::On => true,
            SwitchState::Off => false,
        }
    }
}

#[derive(Args, Debug)]
pub struct UIArgs {
    #[command(subcommand)]
    pub arg: UICommand,
}

#[derive(Subcommand, Debug)]
pub enum UICommand {
    /// get UI element, states, and values
    Get,
    /// set a UI element's value
    Set { selector: String, value: String },
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
    /// List zones
    #[command(alias = "zns")]
    Zones,
    /// List devices in zone
    #[command(alias = "devs")]
    Devices { zone: String },
    /// List subdevices in device
    #[command(alias = "subdevs")]
    Subdevices { dev: String },
    /// List scripts running
    Scripts,
}

impl ListItems {
    pub fn get_selection(&self) -> Option<&str> {
        Some(match self {
            ListItems::Devices { zone } => &zone,
            ListItems::Subdevices { dev } => &dev,
            _ => return None,
        })
    }
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
    /// View device logs
    #[command(alias = "dev")]
    Device { name: String },
    /// View automation logs
    #[command(alias = "atm")]
    Script { name: String },
}

#[derive(Args, Debug)]
pub struct ScriptArgs {
    #[command(subcommand)]
    pub action: ScriptAction,
}

#[derive(Subcommand, Debug)]
pub enum ScriptAction {
    /// Run the script
    Run {
        /// Name of the script
        name: String,
        /// Script arguments
        #[arg(trailing_var_arg = true)]
        extra_args: Vec<String>,
    },
    /// Cancel script instance by ID
    Cancel {
        id: u32
    },
    /// Cancel all instances of this script
    CancelAll {
        name: String
    }
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
