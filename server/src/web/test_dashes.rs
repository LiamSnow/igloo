use axum::extract::ws::Message;
use igloo_interface::{
    ComponentType, DeviceID, QueryFilter, QueryTarget,
    dash::{
        ColorPickerElement, ColorPickerVariant, DashQuery, DashQueryNoType, Dashboard, HAlign,
        HStackElement, Size, SliderElement, SwitchElement, VAlign, VStackElement,
    },
    ws::{ElementUpdate, ServerMessage},
};
use rustc_hash::FxHashMap;
use std::{collections::HashMap, error::Error};
use tokio::sync::mpsc;

use crate::{
    DashboardRequest, GlobalState, glacier::query::WatchAllQuery, web::watch::GetWatchers,
};

pub async fn init_dash(
    state: GlobalState,
    dash_idx: u16,
    dash_id: String,
    mut dash: Dashboard,
) -> Result<(), Box<dyn Error>> {
    dash.idx = Some(dash_idx);
    let watchers = dash.attach_watchers(dash_idx).unwrap();

    let (watch_tx, mut watch_rx) = mpsc::channel(10);
    for watcher in watchers {
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

    let mut dashs = state.dashs.write().await;
    dashs.insert(dash_id, dash);
    drop(dashs);

    tokio::spawn(async move {
        // TODO probably need to keep this handle on GlobalState
        // so we can update queries as dashboard is edited

        let mut dash_rx = state.dash_tx.subscribe();
        let mut watch_values = FxHashMap::default();

        loop {
            tokio::select! {
                Some((watch_id, _, _, value)) = watch_rx.recv() => {
                    watch_values.insert(watch_id, value.clone());
                    let msg: ServerMessage = ElementUpdate { watch_id, value }.into();
                    let bytes = match borsh::to_vec(&msg) {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("Failed to borsh serialize ServerMessage: {msg:?}, because {e}");
                            continue;
                        }
                    };
                    let res = state.cast.send((dash_idx, Message::Binary(bytes.into())));
                    if let Err(e) = res {
                        eprintln!("failed to broadcast: {e}");
                    }
                }

                Ok((req_dash_idx, req)) = dash_rx.recv() => {
                    if req_dash_idx != dash_idx {
                        continue;
                    }

                    match req {
                        DashboardRequest::Shutdown => {
                            break;
                        },
                        DashboardRequest::DumpData => {
                            for (watch_id, value) in watch_values.clone() {
                                let msg: ServerMessage = ElementUpdate { watch_id, value }.into();
                                let bytes = match borsh::to_vec(&msg) {
                                    Ok(b) => b,
                                    Err(e) => {
                                        eprintln!("Failed to borsh serialize ServerMessage: {msg:?}, because {e}");
                                        continue;
                                    }
                                };
                                let res = state.cast.send((dash_idx, Message::Binary(bytes.into())));
                                if let Err(e) = res {
                                    eprintln!("failed to broadcast: {e}");
                                }
                            }
                        },
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn make(state: &GlobalState) -> Result<(), Box<dyn Error>> {
    let mut targets = HashMap::default();
    targets.insert(
        "surf".to_string(),
        QueryTarget::Device(DeviceID::from_parts(0, 0)),
    );

    let main_dash = Dashboard {
        display_name: "Main".to_string(),
        targets,
        idx: None,
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

    let mut targets = HashMap::default();
    targets.insert(
        "surf".to_string(),
        QueryTarget::Device(DeviceID::from_parts(0, 0)),
    );

    let main2_dash = Dashboard {
        display_name: "Main 2".to_string(),
        targets,
        idx: None,
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

    init_dash(state.clone(), 0, "main".into(), main_dash).await?;
    init_dash(state.clone(), 1, "main2".into(), main2_dash).await?;

    Ok(())
}
