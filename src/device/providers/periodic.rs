use std::{cmp::min, sync::Arc};

use jiff::{civil::Time, Span, Zoned};
use serde::{Deserialize, Serialize};
use tokio::{sync::{mpsc, oneshot}, time::Duration};
use tracing::{debug, error, span, Level};

use crate::{
    cli::model::Cli,
    entity::{
        weekly::{Weekly, WeeklyState},
        EntityCommand, EntityState, TargetedEntityCommand,
    }, state::IglooState,
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
    Time {
        default: Time,
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
    istate_rx: oneshot::Receiver<Arc<IglooState>>,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) -> Result<(), String> {
    let span = span!(Level::INFO, "Device Periodic", s=selector, did);
    let _enter = span.enter();
    debug!("initializing");

    let res = on_change_tx.send((did, "connected".to_string(), EntityState::Connection(true))).await;
    if let Err(e) = res {
        error!("sending on_change: {e}");
    }

    let on_trigger = parse_cmd(config.on_trigger)?;
    let on_change = config.on_change.map(parse_cmd).transpose()?;

    let mut notify_change = true;
    let mut weekly = match config.r#type {
        TaskType::Time { default } => Weekly::all_days(default),
        TaskType::Weekly { default } => default,
    };

    let istate = istate_rx.await.unwrap();
    debug!("ready");

    loop {
        if notify_change {
            if let Err(e) = on_change_tx
                .send((did, ENTITY_NAME.to_string(), WeeklyState::from(weekly.clone()).into()))
                .await
            {
                error!("sending on_change: {e}");
            }

            if let Some(ref cmd) = on_change {
                if let Err(e) = cmd.clone().dispatch(&istate, None, true).await {
                    error!("executing on_change cmd: {}", serde_json::to_string(&e).unwrap());
                }
            }
        }

        let sleep_seconds = calc_next_trigger(&weekly) + config.trigger_offset.unwrap_or(0) as i64;

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(sleep_seconds as u64)) => {
                if let Err(e) = on_trigger.clone().dispatch(&istate, None, true).await {
                    error!("executing on_trigger cmd: {}", serde_json::to_string(&e).unwrap());
                }
                notify_change = false;
            }

            Some(cmd) = cmd_rx.recv() => {
                match cmd.cmd {
                    EntityCommand::Weekly(wkly_cmd) => {
                        weekly.apply_cmd(wkly_cmd);
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

fn calc_next_trigger(weekly: &Weekly) -> i64 {
    let now = Zoned::now();
    let cur_day = now.weekday();
    let cur_time = now.time();

    let mut days_until_next = 7;
    for day in &weekly.days {
        let days_until = cur_day.until(*day);
        match days_until {
            0 if cur_time < weekly.time => days_until_next = 0,
            0 => {},
            _ => days_until_next = min(days_until_next, days_until)
        }
    }

    let trigger = now.time_zone().to_zoned(
            now.date()
                .saturating_add(Span::new().days(days_until_next))
                .to_datetime(weekly.time)
        )
        .unwrap();

    now.until(&trigger).unwrap().get_seconds() //FIXME
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
