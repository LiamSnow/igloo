use std::error::Error;

use chrono::{NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::entity::{time::{deserialize_time, serialize_time}, EntityState, TargetedEntityCommand};

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
    Time { default: NaiveTime },
    Weekly { default: Weekly },
    MultiplyWeekly { default: MultipleWeekly },
    DateTime { default: NaiveDateTime }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Weekly {
    pub day: Weekday,
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub time: NaiveTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MultipleWeekly {
    pub days: Vec<Weekday>,
    #[serde(
        deserialize_with = "deserialize_time",
        serialize_with = "serialize_time"
    )]
    pub time: NaiveTime,
}

pub async fn task(
    config: DeviceConfig,
    did: usize,
    _selector: String,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {

    while let Some(msg) = cmd_rx.recv().await {
        //TODO cancel current schedule and make new one with new time
    }
}
