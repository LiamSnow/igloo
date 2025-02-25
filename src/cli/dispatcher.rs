use std::{sync::Arc, time::Duration};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;
use tokio::{sync::{mpsc, oneshot}, time};

use crate::{command::{Color, SubdeviceCommand}, map::IglooStack, selector::{DevCommandChannelRef, OwnedSelection, Selection, SelectorError}, VERSION};

use super::model::{Cli, CliCommands, LightAction, LightEffect, ListItems, LogType};

#[derive(Error, Debug, Serialize)]
pub enum DispatchError {
    #[error("selector error `{0}`")]
    SelectorError(SelectorError),
    #[error("unknown zone `{0}`")]
    UnknownZone(String),
}

impl From<SelectorError> for DispatchError {
    fn from(value: SelectorError) -> Self {
        Self::SelectorError(value)
    }
}

impl Cli {
    pub async fn dispatch(self, stack: Arc<IglooStack>) -> Result<impl IntoResponse, DispatchError> {
        Ok(match self.command {
            CliCommands::Light(args) => {
                let selection = Selection::new(&args.target)?;

                //Cancel any lighting effects running on this light
                {
                    //TODO optimization
                    let mut cur_effects = stack.current_effects.lock().await;
                    for i in 0..cur_effects.len() {
                        let (eff_selection, _) = cur_effects.get(i).unwrap();
                        if eff_selection.collides(&selection) {
                            println!("Effect cancelled!");
                            let (_, cancel_tx) = cur_effects.remove(i);
                            cancel_tx.send(true).unwrap(); //FIXME
                        }
                    }
                }

                let mut devs = DevCommandChannelRef::new(&stack.cmd_chan_map, &selection)?;
                devs.execute(SubdeviceCommand::Light(args.action))?;
                (StatusCode::OK).into_response()
            },
            CliCommands::Effect(args) => {
                let selection: OwnedSelection = Selection::new(&args.target)?.into();
                let selection_ref = (&selection).into();
                let mut devs = DevCommandChannelRef::new(&stack.cmd_chan_map, &selection_ref)?;

                let (cancel_tx, mut cancel_rx) = oneshot::channel::<bool>();
                {
                    let mut cur_effects = stack.current_effects.lock().await;

                    //Cancel any conflicting effects
                    for i in 0..cur_effects.len() {
                        let (eff_selection, _) = cur_effects.get(i).unwrap();
                        if eff_selection.collides(&selection_ref) {
                            println!("Effect cancelled!");
                            let (_, cancel_tx) = cur_effects.remove(i);
                            cancel_tx.send(true).unwrap(); //FIXME
                        }
                    }

                    cur_effects.push((selection, cancel_tx));
                }

                match args.effect {
                    LightEffect::BrightnessFade { start_brightness, end_brightness, length_ms } => {
                        tokio::spawn(async move {
                            let step_ms = 50;
                            let num_steps = length_ms / step_ms;
                            let step_brightness = ((end_brightness - start_brightness) as f32) / num_steps as f32;
                            let mut brightness = start_brightness as f32;
                            let mut interval = time::interval(Duration::from_millis(step_ms.into()));
                            for _ in 0..num_steps {
                                interval.tick().await;
                                if cancel_rx.try_recv().is_ok() {
                                    break;
                                }
                                devs.execute(LightAction::Brightness { brightness: brightness as u8 }.into()).unwrap(); //FIXME
                                brightness += step_brightness;
                            }
                        });
                    },
                    LightEffect::Rainbow { speed, length_ms } => {
                        tokio::spawn(async move {
                            //255 => 1000ms
                            //0 => 10ms
                            let step_ms = (((255 - speed) as f32 / 255.) * 100. + 10.) as u32;
                            let mut interval = time::interval(Duration::from_millis(step_ms.into()));
                            let mut hue = 0;
                            let num_steps = length_ms.and_then(|l| Some(l / step_ms));
                            let mut step_num = 0;
                            loop {
                                interval.tick().await;
                                if cancel_rx.try_recv().is_ok() {
                                    println!("Effect recieved shutdown");
                                    break;
                                }
                                devs.execute(LightAction::Color(Color::from_hue8(hue)).into()).unwrap(); //FIXME
                                hue = (hue + 1) % 255;
                                if let Some(num_steps) = num_steps {
                                    step_num += 1;
                                    if step_num > num_steps {
                                        break;
                                    }
                                }
                            }
                            println!("Effect shutdown");
                        });
                    }
                }

                (StatusCode::OK).into_response()
            },
            CliCommands::Switch(_) => todo!(),
            CliCommands::UI => {
                (StatusCode::OK, Json(&stack.ui_elements)).into_response()
            }
            CliCommands::List(args) => {
                match args.item {
                    ListItems::Users => todo!(),
                    ListItems::UserGroups => todo!(),
                    ListItems::Providers => todo!(),
                    ListItems::Automations => todo!(),
                    ListItems::Zones => {
                        let zones: Vec<_> = stack.cmd_chan_map.keys().collect();
                        (StatusCode::OK, Json(zones)).into_response()
                    },
                    ListItems::Devices { zone } => {
                        let mut dev_names = Vec::new();
                        let zone = stack.cmd_chan_map.get(&zone).ok_or(DispatchError::UnknownZone(zone))?;
                        for (dev_name, _) in zone {
                            dev_names.push(dev_name.to_string());
                        }
                        (StatusCode::OK, Json(dev_names)).into_response()
                    },
                    ListItems::Subdevices { dev: _ } => {
                        todo!()
                    },
                }
            },
            CliCommands::Logs(args) => {
                match args.log_type {
                    LogType::System => todo!(),
                    LogType::User { user: _ } => todo!(),
                    LogType::Device { dev: _ } => {
                        todo!()
                    },
                    LogType::Automation { automation: _ } => todo!(),
                }
            },
            CliCommands::Automation(_) => todo!(),
            CliCommands::Reload => todo!(),
            CliCommands::Version => (StatusCode::OK, Json(VERSION)).into_response()
        })
    }
}

