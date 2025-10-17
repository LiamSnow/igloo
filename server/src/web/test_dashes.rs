use axum::extract::ws::Message;
use igloo_interface::{
    ComponentType, DeviceID, QueryFilter, QueryTarget,
    dash::{
        ColorPickerElement, ColorPickerVariant, DashQuery, DashQueryNoType, Dashboard, HAlign,
        HStackElement, Size, SliderElement, SwitchElement, VAlign, VStackElement,
    },
    ws::{ElementUpdate, ServerMessage},
};
use std::{collections::HashMap, error::Error};
use tokio::sync::mpsc;

use crate::{GlobalState, glacier::query::WatchAllQuery, web::watch::GetWatchers};

pub async fn make(state: GlobalState) -> Result<(), Box<dyn Error>> {
    let mut targets = HashMap::default();
    targets.insert(
        "surf".to_string(),
        QueryTarget::Device(DeviceID::from_parts(0, 0)),
    );

    let dash_idx = 0;

    let mut dash = Dashboard {
        display_name: "Main".to_string(),
        targets,
        idx: Some(dash_idx),
        child: VStackElement {
            justify: VAlign::Center,
            align: HAlign::Center,
            scroll: false,
            children: vec![
                SliderElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::Dimmer,
                    },
                    auto_validate: false,
                    min: Some(0.),
                    max: Some(1.),
                    step: None,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::ColorWheel,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::Square,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::HueSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::SaturationSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::RedSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::GreenSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::BlueSlider,
                }
                .into(),
                SliderElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::ColorTemperature,
                    },
                    auto_validate: false,
                    min: Some(2000.),
                    max: Some(7000.),
                    step: Some(1.),
                }
                .into(),
                SwitchElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::Switch,
                    },
                    size: Size::Medium,
                }
                .into(),
            ],
        }
        .into(),
    };

    let watchers = dash.attach_watchers(dash_idx).unwrap();

    let (watch_tx, mut watch_rx) = mpsc::channel(10);
    for watcher in watchers {
        println!("registering query");
        state
            .query_tx
            .send(
                WatchAllQuery {
                    filter: watcher.filter,
                    target: watcher.target,
                    update_tx: watch_tx.clone(),
                    comp: watcher.comp,
                    prefix: watcher.watch_id,
                }
                .into(),
            )
            .await
            .unwrap();
    }
    let gs = state.clone();
    tokio::spawn(async move {
        while let Some((watch_id, _, _, value)) = watch_rx.recv().await {
            // TODO we need some system of collecting all of these,
            // then shipping out all values to new viewers
            let msg: ServerMessage = ElementUpdate { watch_id, value }.into();
            let bytes = borsh::to_vec(&msg).unwrap(); // FIXME unwrap
            let dash_idx = (watch_id >> 16) as u16;
            let res = gs.cast.send((dash_idx, Message::Binary(bytes.into())));
            if let Err(e) = res {
                eprintln!("failed to broadcast: {e}");
            }
        }
    });

    let mut dashs = state.dashboards.write().await;
    dashs.insert("main".into(), dash);
    drop(dashs);

    let mut targets = HashMap::default();
    targets.insert(
        "surf".to_string(),
        QueryTarget::Device(DeviceID::from_parts(0, 0)),
    );

    let dash_idx = 1;

    let mut dash = Dashboard {
        display_name: "Main 2".to_string(),
        targets,
        idx: Some(dash_idx),
        child: HStackElement {
            justify: HAlign::Center,
            align: VAlign::Center,
            scroll: false,
            children: vec![
                SliderElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::Dimmer,
                    },
                    auto_validate: false,
                    min: Some(0.),
                    max: Some(1.),
                    step: None,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::ColorWheel,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::Square,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::HueSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::SaturationSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::RedSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::GreenSlider,
                }
                .into(),
                ColorPickerElement {
                    watch_id: None,
                    binding: DashQueryNoType {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                    },
                    variant: ColorPickerVariant::BlueSlider,
                }
                .into(),
                SliderElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::ColorTemperature,
                    },
                    auto_validate: false,
                    min: Some(2000.),
                    max: Some(7000.),
                    step: Some(1.),
                }
                .into(),
                SwitchElement {
                    watch_id: None,
                    binding: DashQuery {
                        target: "surf".to_string(),
                        filter: QueryFilter::With(ComponentType::Light),
                        comp_type: ComponentType::Switch,
                    },
                    size: Size::Medium,
                }
                .into(),
            ],
        }
        .into(),
    };

    let watchers = dash.attach_watchers(dash_idx).unwrap();

    let (watch_tx, mut watch_rx) = mpsc::channel(10);
    for watcher in watchers {
        println!("registering query");
        state
            .query_tx
            .send(
                WatchAllQuery {
                    filter: watcher.filter,
                    target: watcher.target,
                    update_tx: watch_tx.clone(),
                    comp: watcher.comp,
                    prefix: watcher.watch_id,
                }
                .into(),
            )
            .await
            .unwrap();
    }
    let gs = state.clone();
    tokio::spawn(async move {
        while let Some((watch_id, _, _, value)) = watch_rx.recv().await {
            // TODO we need some system of collecting all of these,
            // then shipping out all values to new viewers
            let msg: ServerMessage = ElementUpdate { watch_id, value }.into();
            let bytes = borsh::to_vec(&msg).unwrap(); // FIXME unwrap
            let dash_idx = (watch_id >> 16) as u16;
            let res = gs.cast.send((dash_idx, Message::Binary(bytes.into())));
            if let Err(e) = res {
                eprintln!("failed to broadcast: {e}");
            }
        }
    });

    let mut dashs = state.dashboards.write().await;
    dashs.insert("main_2".into(), dash);
    drop(dashs);

    Ok(())
}
