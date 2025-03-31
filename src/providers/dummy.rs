use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{cli::model::Cli, entity::{
    bool::BoolState,
    float::FloatState,
    int::IntState,
    text::TextState,
    time::{deserialize_time, serialize_time, TimeState},
    EntityCommand, EntityState, TargetedEntityCommand,
}};

const ENTITY_NAME: &str = "value";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceConfig {
    pub r#type: VarType,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum VarType {
    Int {
        default: i32,
        range: Option<(i32, i32)>,
    },
    Float {
        default: f32,
        range: Option<(f32, f32)>,
    },
    Bool {
        default: bool,
    },
    Text {
        default: String,
    },
    Time {
        #[serde(
            deserialize_with = "deserialize_time",
            serialize_with = "serialize_time"
        )]
        default: NaiveTime,
    },
}

pub async fn task(
    config: DeviceConfig,
    did: usize,
    _selector: String,
    _cmd_tx: mpsc::Sender<Cli>,
    cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    match config.r#type {
        VarType::Int { default, range } => {
            int_task(default, range, did, cmd_rx, on_change_tx).await
        }
        VarType::Float { default, range } => {
            float_task(default, range, did, cmd_rx, on_change_tx).await
        }
        VarType::Bool { default } => bool_task(default, did, cmd_rx, on_change_tx).await,
        VarType::Text { default } => text_task(default, did, cmd_rx, on_change_tx).await,
        VarType::Time { default } => time_task(default, did, cmd_rx, on_change_tx).await,
    }
}

pub async fn bool_task(
    default: bool,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    // send init state
    let res = on_change_tx
        .send((
            did,
            ENTITY_NAME.to_string(),
            BoolState::from(default).into(),
        ))
        .await;
    if let Err(e) = res {
        println!("Dummy error sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Bool(value) = cmd.cmd {
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), value.into()))
                .await;
            if let Err(e) = res {
                println!("Dummy error sending on_change: {e}");
            }
        } else {
            println!("Dummy invalid entity command type");
        }
    }
}

pub async fn int_task(
    default: i32,
    range: Option<(i32, i32)>,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    // send init state
    let res = on_change_tx
        .send((did, ENTITY_NAME.to_string(), IntState::from(default).into()))
        .await;
    if let Err(e) = res {
        println!("Dummy error sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Int(value) = cmd.cmd {
            if let Some((min, max)) = range {
                if value < min || value > max {
                    println!("Dummy int out of range (value:{value},min:{min}:max{max}). Skipping");
                    continue;
                }
            }

            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), IntState::from(value).into()))
                .await;
            if let Err(e) = res {
                println!("Dummy error sending on_change: {e}");
            }
        } else {
            println!("Dummy invalid entity command type");
        }
    }
}

pub async fn float_task(
    default: f32,
    range: Option<(f32, f32)>,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    // send init state
    let res = on_change_tx
        .send((
            did,
            ENTITY_NAME.to_string(),
            FloatState::from(default).into(),
        ))
        .await;
    if let Err(e) = res {
        println!("Dummy error sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Float(value) = cmd.cmd {
            if let Some((min, max)) = range {
                if value < min || value > max {
                    println!(
                        "Dummy float out of range (value:{value},min:{min}:max{max}). Skipping"
                    );
                    continue;
                }
            }

            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), FloatState::from(value).into()))
                .await;
            if let Err(e) = res {
                println!("Dummy error sending on_change: {e}");
            }
        } else {
            println!("Dummy invalid entity command type");
        }
    }
}

pub async fn text_task(
    default: String,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    // send init state
    let res = on_change_tx
        .send((
            did,
            ENTITY_NAME.to_string(),
            TextState::from(default).into(),
        ))
        .await;
    if let Err(e) = res {
        println!("Dummy error sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Text(value) = cmd.cmd {
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), TextState::from(value).into()))
                .await;
            if let Err(e) = res {
                println!("Dummy error sending on_change: {e}");
            }
        } else {
            println!("Dummy invalid entity command type");
        }
    }
}

pub async fn time_task(
    default: NaiveTime,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    // send init state
    let res = on_change_tx
        .send((
            did,
            ENTITY_NAME.to_string(),
            TimeState::from(default).into(),
        ))
        .await;
    if let Err(e) = res {
        println!("Dummy error sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Time(value) = cmd.cmd {
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), TimeState::from(value).into()))
                .await;
            if let Err(e) = res {
                println!("Dummy error sending on_change: {e}");
            }
        } else {
            println!("Dummy invalid entity command type");
        }
    }
}
