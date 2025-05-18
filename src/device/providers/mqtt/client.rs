use std::{collections::HashMap, sync::Arc, time::Duration};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, span, Level};
use uuid::Uuid;

use crate::{
    cli::model::Cli,
    device::providers::periodic::parse_cmd,
    entity::{EntityCommand, EntityState, TargetedEntityCommand},
    state::IglooState,
};

use super::{DeviceConfig, ResponsePlan};

enum ParsedResponsePlan {
    RespondWith(String, String),
    RespondWithCommandResult(Cli, String),
    RunCommand(Cli),
}

// TODO reconnect logic?
pub async fn task(
    cfg: DeviceConfig,
    did: usize,
    selector: String,
    istate_rx: oneshot::Receiver<Arc<IglooState>>,
    mut cmd_rx: mpsc::Receiver<TargetedEntityCommand>,
    _on_change_tx: mpsc::Sender<(usize, String, EntityState)>,
) -> Result<(), String> {
    let span = span!(Level::INFO, "Device MQTT", s = selector, did);
    let _enter = span.enter();
    debug!("initializing");

    let istate = istate_rx.await.unwrap();

    // init
    let client_id = format!("igloo_{}", Uuid::now_v7());
    let mut mqttoptions = MqttOptions::new(client_id, cfg.host, cfg.port);
    mqttoptions.set_credentials(cfg.username, cfg.password);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // parse cmds & subscribe
    let mut subs = HashMap::with_capacity(cfg.subscribe.len());
    for sub in cfg.subscribe {
        client
            .subscribe(&sub.0, QoS::AtLeastOnce)
            .await
            .map_err(|e| e.to_string())?;
        subs.insert(sub.0, ParsedResponsePlan::parse(sub.1)?);
    }

    //TODO notify connected

    let client_clone = client.clone();
    tokio::spawn(async move {
        while let Ok(notif) = eventloop.poll().await {
            if let Event::Incoming(Packet::Publish(publish)) = notif {
                let plan = match subs.get(&publish.topic) {
                    Some(p) => p,
                    None => {
                        error!("Unhandled sub {:#?}", publish);
                        continue;
                    }
                };

                let res = plan.get_response(&istate).await;

                match res {
                    Ok(Some(resp)) => {
                        let res = client_clone
                            .publish(plan.get_dest_path().unwrap(), QoS::AtLeastOnce, false, resp)
                            .await;
                        if let Err(e) = res {
                            error!("sending response: {e}");
                        }
                    }
                    Ok(None) => {}
                    Err(e) => error!("{e}"),
                }
            }
        }
    });

    while let Some(cmd) = cmd_rx.recv().await {
        let pub_cfg = match cfg.publish.get(&cmd.entity_name.unwrap()) {
            Some(r) => r,
            None => {
                error!("unknown entity");
                continue
            },
        };

        if pub_cfg.get_entity_type() != cmd.cmd.get_type() {
            error!("expected {:#?}, got {:#?}", pub_cfg.get_entity_type(), cmd.cmd.get_type());
            continue
        }

        let payload = match cmd.cmd {
            EntityCommand::Int(v) => &v.to_string(),
            EntityCommand::Float(v) => &v.to_string(),
            EntityCommand::Bool(v) => &v.to_string(),
            EntityCommand::Text(ref v) => v,
            EntityCommand::Trigger => "",
            _ => panic!()
        };

        let res = client
            .publish(pub_cfg.get_path(), QoS::ExactlyOnce, false, payload)
            .await;

        if let Err(e) = res {
            error!("publishing {e}")
        }
    }

    Ok(())
}

impl ParsedResponsePlan {
    fn parse(plan: ResponsePlan) -> Result<Self, String> {
        Ok(match plan {
            ResponsePlan::RespondWith(v, p) => Self::RespondWith(v, p),
            ResponsePlan::RespondWithCommandResult(c, p) => {
                Self::RespondWithCommandResult(parse_cmd(c)?, p)
            }
            ResponsePlan::RunCommand(c) => Self::RunCommand(parse_cmd(c)?),
        })
    }

    fn get_cmd(&self) -> Option<Cli> {
        match self {
            Self::RespondWith(_, _) => None,
            Self::RespondWithCommandResult(cli, _) => Some(cli.clone()),
            Self::RunCommand(cli) => Some(cli.clone()),
        }
    }

    fn get_dest_path(&self) -> Option<&str> {
        match self {
            Self::RespondWith(_, p) => Some(p),
            Self::RespondWithCommandResult(_, p) => Some(p),
            Self::RunCommand(..) => None,
        }
    }

    async fn get_response(&self, istate: &Arc<IglooState>) -> Result<Option<String>, String> {
        let res = match self.get_cmd() {
            Some(cmd) => match cmd.clone().dispatch(&istate, None, true).await {
                Ok(r) => r,
                Err(e) => {
                    return Err(format!(
                        "executing cmd: {}",
                        serde_json::to_string(&e).unwrap()
                    ))
                }
            },
            None => None,
        };

        Ok(match self {
            Self::RespondWith(v, _) => Some(v.to_string()),
            Self::RespondWithCommandResult(..) => res,
            Self::RunCommand(..) => None,
        })
    }
}
