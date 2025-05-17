use jiff::civil::{DateTime, Time};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, span, Level};

use crate::{
    cli::model::Cli,
    entity::{
        bool::BoolState,
        datetime::DateTimeState,
        float::FloatState,
        int::IntState,
        text::TextState,
        time::TimeState,
        weekly::{Weekly, WeeklyState},
        EntityCommand, EntityState, TargetedEntityCommand,
    },
};

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
        default: Time,
    },
    DateTime {
        default: DateTime,
    },
    Weekly {
        default: Weekly,
    },
}

pub async fn task(
    config: DeviceConfig,
    did: usize,
    selector: String,
    _cmd_tx: mpsc::Sender<Cli>,
    cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    let span = span!(Level::DEBUG, "Device Dummy", s = selector, did);
    let _enter = span.enter();
    debug!("initializing");

    let res = on_change_tx.send((did, "connected".to_string(), EntityState::Connection(true))).await;
    if let Err(e) = res {
        error!("sending on_change: {e}");
    }

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
        VarType::DateTime { default } => datetime_task(default, did, cmd_rx, on_change_tx).await,
        VarType::Weekly { default } => weekly_task(default, did, cmd_rx, on_change_tx).await,
    }
}

// TODO -- reduce all this repeated code plz

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
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Bool(value) = cmd.cmd {
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), value.into()))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!("dummy expected type Bool, found {:#?}", cmd.cmd.get_type());
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
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Int(value) = cmd.cmd {
            if let Some((min, max)) = range {
                if value < min || value > max {
                    error!("int out of range (value:{value},min:{min}:max{max}). Skipping");
                    continue;
                }
            }

            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), IntState::from(value).into()))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!("dummy expected type Int, found {:#?}", cmd.cmd.get_type());
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
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Float(value) = cmd.cmd {
            if let Some((min, max)) = range {
                if value < min || value > max {
                    error!("float out of range (value:{value},min:{min}:max{max}). Skipping");
                    continue;
                }
            }

            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), FloatState::from(value).into()))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!("dummy expected type Float, found {:#?}", cmd.cmd.get_type());
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
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Text(value) = cmd.cmd {
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), TextState::from(value).into()))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!("dummy expected type Text, found {:#?}", cmd.cmd.get_type());
        }
    }
}

pub async fn time_task(
    default: Time,
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
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Time(value) = cmd.cmd {
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), TimeState::from(value).into()))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!("dummy expected type Time, found {:#?}", cmd.cmd.get_type());
        }
    }
}

pub async fn datetime_task(
    default: DateTime,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    // send init state
    let res = on_change_tx
        .send((
            did,
            ENTITY_NAME.to_string(),
            DateTimeState::from(default).into(),
        ))
        .await;
    if let Err(e) = res {
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::DateTime(value) = cmd.cmd {
            let res = on_change_tx
                .send((
                    did,
                    ENTITY_NAME.to_string(),
                    DateTimeState::from(value).into(),
                ))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!(
                "dummy expected type DateTime, found {:#?}",
                cmd.cmd.get_type()
            );
        }
    }
}

pub async fn weekly_task(
    default: Weekly,
    did: usize,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) {
    let mut state = WeeklyState::from(default);

    // send init state
    let res = on_change_tx
        .send((did, ENTITY_NAME.to_string(), state.clone().into()))
        .await;
    if let Err(e) = res {
        error!("sending on_change: {e}");
    }

    while let Some(cmd) = cmd_rx.recv().await {
        if let EntityCommand::Weekly(wkly_cmd) = cmd.cmd {
            state.apply_cmd(wkly_cmd);
            let res = on_change_tx
                .send((did, ENTITY_NAME.to_string(), state.clone().into()))
                .await;
            if let Err(e) = res {
                error!("sending on_change: {e}");
            }
        } else {
            error!(
                "dummy expected type Weekly, found {:#?}",
                cmd.cmd.get_type()
            );
        }
    }
}
