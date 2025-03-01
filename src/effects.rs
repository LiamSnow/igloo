use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Serialize;
use tokio::{sync::oneshot, time};

use crate::{
    cli::model::{LightAction, LightEffect},
    command::Color,
    map::IglooStack,
    selector::Selection,
};

#[derive(Default)]
pub struct EffectsState {
    pub next_id: u32,
    pub current: HashMap<u32, EffectMeta>,
}

pub struct EffectMeta {
    claim: LightClaim,
    effect: LightEffect,
    sel: Selection,
    cancel_tx: Option<oneshot::Sender<()>>,
}

#[derive(Serialize)]
pub struct EffectDisplay {
    id: u32,
    effect: LightEffect,
    selection: String,
}

pub async fn clear_conflicting(stack: &Arc<IglooStack>, selection: &Selection, claim: &LightClaim) {
    let mut effects = stack.effects_state.lock().await;
    for (_, meta) in &mut effects.current {
        if meta.cancel_tx.is_some() {
            if claim.collides(&meta.claim) && selection.collides(&meta.sel) {
                let _ = meta.cancel_tx.take().unwrap().send(());
            }
        }
    }
}

pub async fn spawn(stack: Arc<IglooStack>, selection: Selection, effect: LightEffect) {
    let claim = (&effect).into();
    clear_conflicting(&stack, &selection, &claim).await;
    let (cancel_tx, mut cancel_rx) = oneshot::channel();

    let id;
    {
        let mut effects = stack.effects_state.lock().await;
        id = effects.next_id;
        effects.current.insert(
            id,
            EffectMeta {
                effect: effect.clone(),
                claim,
                sel: selection.clone(),
                cancel_tx: Some(cancel_tx),
            },
        );
        effects.next_id += 1;
    }

    match effect {
        LightEffect::Cancel => return,
        LightEffect::BrightnessFade {
            start_brightness,
            end_brightness,
            length_ms,
        } => {
            tokio::spawn(async move {
                let step_ms = 50;
                let num_steps = length_ms / step_ms;
                let step_brightness =
                    ((end_brightness - start_brightness) as f32) / num_steps as f32;
                let mut brightness = start_brightness as f32;
                let mut interval = time::interval(Duration::from_millis(step_ms.into()));
                for _ in 0..num_steps {
                    interval.tick().await;
                    if cancel_rx.try_recv().is_ok() {
                        break;
                    }

                    selection
                        .execute(
                            &stack,
                            LightAction::Brightness {
                                brightness: brightness as u8,
                            }
                            .into(),
                        )
                        .unwrap(); //FIXME
                    brightness += step_brightness;
                }
                println!("00 brightnessfade effect shutdown");
                let mut effects = stack.effects_state.lock().await;
                effects.current.remove(&id);
            })
        }
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
                        break;
                    }

                    selection
                        .execute(&stack, LightAction::Color(Color::from_hue8(hue)).into())
                        .unwrap(); //FIXME
                    hue = (hue + 1) % 255;
                    if let Some(num_steps) = num_steps {
                        step_num += 1;
                        if step_num > num_steps {
                            break;
                        }
                    }
                }
                println!("00 rainbow effect shutdown");
                let mut effects = stack.effects_state.lock().await;
                effects.current.remove(&id);
            })
        }
    };
}

pub async fn list(stack: &Arc<IglooStack>, selection: &Selection) -> Vec<EffectDisplay> {
    let mut res = Vec::new();
    let mut effects = stack.effects_state.lock().await;
    for (id, meta) in &mut effects.current {
        if selection.collides(&meta.sel) {
            res.push(EffectDisplay {
                id: *id,
                effect: meta.effect.clone(),
                selection: meta.sel.to_str(&stack.lut),
            });
        }
    }
    res
}

#[derive(PartialEq, Eq)]
pub enum LightClaim {
    All,
    ColorOrTemp,
    Brightness,
}

impl LightClaim {
    fn collides(&self, other: &Self) -> bool {
        matches!(self, Self::All) || matches!(other, Self::All) || self == other
    }
}

impl From<&LightAction> for LightClaim {
    fn from(value: &LightAction) -> Self {
        match value {
            LightAction::On | LightAction::Off => Self::All,
            LightAction::Color(_) => Self::ColorOrTemp,
            LightAction::Temperature { .. } => Self::ColorOrTemp,
            LightAction::Brightness { .. } => Self::Brightness,
        }
    }
}

impl From<&LightEffect> for LightClaim {
    fn from(value: &LightEffect) -> Self {
        match value {
            LightEffect::Cancel => Self::All,
            LightEffect::BrightnessFade { .. } => Self::Brightness,
            LightEffect::Rainbow { .. } => Self::ColorOrTemp,
        }
    }
}
