use std::cmp::{max, min};

use chrono::{Duration, Local, NaiveTime, Datelike};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{error, info, span, Level};

use crate::{
    cli::model::Cli,
    entity::{
        time::{deserialize_time, serialize_time},
        weekly::{Weekly, WeeklyState},
        EntityCommand, EntityState, TargetedEntityCommand,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceConfig {
    pub r#type: TaskType,
    pub trigger_offset: Option<i32>,
    pub on_trigger: String,
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
}

const ENTITY_NAME: &str = "value";

pub async fn task(
    config: DeviceConfig,
    did: usize,
    selector: String,
    cmd_tx: mpsc::Sender<Cli>,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) -> Result<(), String> {
    let span = span!(Level::INFO, "Device Periodic", s=selector, did);
    let _enter = span.enter();
    info!("initializing");

    let on_trigger = parse_cmd(config.on_trigger)?;
    let on_change = config.on_change.map(parse_cmd).transpose()?;

    let mut notify_change = true;
    let mut weekly = match config.r#type {
        TaskType::Time { default } => Weekly::all_days(default),
        TaskType::Weekly { default } => default,
    };

    loop {
        if notify_change {
            if let Err(e) = on_change_tx
                .send((did, ENTITY_NAME.to_string(), WeeklyState::from(weekly.clone()).into()))
                .await
            {
                error!("sending on_change: {e}");
            }

            if let Some(ref cmd) = on_change {
                if let Err(e) = cmd_tx.send(cmd.clone()).await {
                    error!("sending on_change command: {e}");
                }
            }
        }

        let now = Local::now().naive_local();
        let next_trigger = calc_next_trigger(&weekly, &now);
        let sleep_seconds = (next_trigger - now).num_seconds() + config.trigger_offset.unwrap_or(0) as i64;
        let sleep_duration = tokio::time::Duration::from_secs(max(0, sleep_seconds) as u64);

        tokio::select! {
            _ = tokio::time::sleep(sleep_duration) => {
                if let Err(e) = cmd_tx.send(on_trigger.clone()).await {
                    error!("sending on_trigger: {e}");
                }
                notify_change = false;
            }

            Some(cmd) = cmd_rx.recv() => {
                match cmd.cmd {
                    EntityCommand::Weekly(new_weekly) => {
                        weekly = new_weekly;
                        notify_change = true;
                    },
                    EntityCommand::Time(new_time) => {
                        weekly = Weekly::all_days(new_time);
                        notify_change = true;
                    },
                    _ => notify_change = false,
                }
            }
        }
    }
}

fn calc_next_trigger(weekly: &Weekly, now: &chrono::NaiveDateTime) -> chrono::NaiveDateTime {
    let cur_day = now.weekday();
    let cur_time = now.time();

    let mut min_days_to_wait = 7;
    for day in &weekly.days {
        let days_to_wait = (day.num_days_from_monday() as i32 - cur_day.num_days_from_monday() as i32 + 7) % 7;

        if days_to_wait == 0 {
            if cur_time < weekly.time {
                min_days_to_wait = 0;
                break;
            } else {
                min_days_to_wait = min(min_days_to_wait, 7);
            }
        } else {
            min_days_to_wait = min(min_days_to_wait, days_to_wait);
        }
    }

    let target_date = now.date() + Duration::days(min_days_to_wait as i64);
    target_date.and_time(weekly.time)
}

pub fn parse_cmd(cmd_str: String) -> Result<Cli, String> {
    Ok(match Cli::parse(&cmd_str) {
        Ok(r) => r,
        Err(e) => {
            return Err(format!(
                "Error parsing periodic task command: {}",
                e.render()
            ))
        }
    })
}
