use chrono::{NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    cli::model::Cli,
    entity::{
        time::{deserialize_time, serialize_time},
        weekly::Weekly,
        weekly_mult::MultipleWeekly,
        EntityState, TargetedEntityCommand,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceConfig {
    pub r#type: TaskType,
    pub trigger_offset: Option<i32>,
    pub on_trigger: Option<String>,
    pub on_change: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskType {
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    Time {
        default: NaiveTime,
    },
    Weekly {
        default: Weekly,
    },
    MultiplyWeekly {
        default: MultipleWeekly,
    },
    DateTime {
        default: NaiveDateTime,
    },
}

pub async fn task(
    config: DeviceConfig,
    did: usize,
    _selector: String,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    let on_trigger = parse_cmd(config.on_trigger);
    let on_change = parse_cmd(config.on_change);

    //TODO make schedule that calls `on_trigger` command every config.type

    while let Some(msg) = cmd_rx.recv().await {
        //TODO cancel current schedule and make new one with new time
        //then call on_change command
    }
}

pub fn parse_cmd(cmd_str_opt: Option<String>) -> Result<Option<Cli>, String> {
    Ok(match cmd_str_opt {
        Some(cmd_str) => match Cli::parse(&cmd_str) {
            Ok(r) => Some(r),
            Err(e) => {
                return Err(format!(
                    "Error parsing period task on_trigger: {}",
                    e.render()
                ))
            }
        },
        None => None,
    })
}
