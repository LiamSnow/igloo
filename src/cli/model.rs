use chrono::NaiveTime;
use clap::command;
use clap::Parser;
use clap::Subcommand;
use clap_derive::{Args, Parser, Subcommand};

use crate::entity::bool::BoolCommand;
use crate::entity::light::LightCommand;
use crate::entity::EntityType;

#[derive(Parser, Debug, Clone)]
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

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
    /// Control lights
    #[command(alias = "lights")]
    Light(SelectorAndAction<LightCommand>),
    Int(IntArgs),
    Float(FloatArgs),
    #[command(alias = "switch")]
    Bool(SelectorAndAction<BoolCommand>),
    Text(TextArgs),
    Time(TimeArgs),

    /// Get UI Interface
    UI,
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
            Self::Int(args) => &args.target,
            Self::Float(args) => &args.target,
            Self::Bool(args) => &args.target,
            Self::Text(args) => &args.target,
            Self::List(args) => return args.item.get_selection(),
            _ => return None,
        })
    }

    pub fn get_entity_type(&self) -> Option<EntityType> {
        Some(match self {
            Self::Light(..) => EntityType::Light,
            Self::Bool(..) => EntityType::Bool,
            _ => return None,
        })
    }
}

#[derive(Args, Debug, Clone)]
pub struct SelectorAndAction<T: Subcommand> {
    /// selector string
    pub target: String,
    #[command(subcommand)]
    pub action: T,
}

#[derive(Args, Debug, Clone)]
pub struct IntArgs {
    /// selector string
    pub target: String,
    pub value: i32,
}

#[derive(Args, Debug, Clone)]
pub struct FloatArgs {
    /// selector string
    pub target: String,
    pub value: f32,
}

#[derive(Args, Debug, Clone)]
pub struct TextArgs {
    /// selector string
    pub target: String,
    pub value: String,
}

#[derive(Args, Debug, Clone)]
pub struct TimeArgs {
    /// selector string
    pub target: String,
    pub value: NaiveTime,
}

#[derive(Args, Debug, Clone)]
pub struct ListArgs {
    #[command(subcommand)]
    pub item: ListItems,
}

#[derive(Subcommand, Debug, Clone)]
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
    /// List entities in device
    Entities { dev: String },
    /// List scripts running
    Scripts,
}

impl ListItems {
    pub fn get_selection(&self) -> Option<&str> {
        Some(match self {
            ListItems::Devices { zone } => &zone,
            ListItems::Entities { dev } => &dev,
            _ => return None,
        })
    }
}

#[derive(Args, Debug, Clone)]
pub struct DescribeArgs {
    #[command(subcommand)]
    pub item: DescribeItems,
}

#[derive(Subcommand, Debug, Clone)]
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
    Script { name: String },
}

#[derive(Args, Debug, Clone)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub log_type: LogType,
}

#[derive(Subcommand, Debug, Clone)]
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

#[derive(Args, Debug, Clone)]
pub struct ScriptArgs {
    #[command(subcommand)]
    pub action: ScriptAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ScriptAction {
    /// Run the script
    Run {
        /// Name of the script
        name: String,
        /// Script arguments
        #[arg(trailing_var_arg = true)]
        extra_args: Vec<String>,
    },
    RunWithId {
        /// Name of the script
        name: String,
        /// ID for the script
        sid: u32,
        /// Script arguments
        #[arg(trailing_var_arg = true)]
        extra_args: Vec<String>,
    },
    /// Cancel script instance by ID
    Cancel { sid: u32 },
    /// Cancel all instances of this script
    CancelAll { name: String },
}
