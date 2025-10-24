use dioxus::prelude::*;
use std::collections::HashSet;

use crate::ws::CURRENT_SNAPSHOT;
use igloo_interface::{Component, DeviceID, DeviceSnapshot, EntitySnapshot, Snapshot};

const TREE_CSS: Asset = asset!("/assets/styling/tree.css");

// TODO should add modification and refreshing/watching

// TODO use dioxus stores so we don't need to be cloning all over the place!!

#[component]
pub fn Tree() -> Element {
    let expanded = use_signal(HashSet::<String>::new);

    match CURRENT_SNAPSHOT.read().cloned() {
        Some(snap) if !snap.devices.is_empty() => {
            rsx! {
                document::Link { rel: "stylesheet", href: TREE_CSS }
                div { class: "tree",
                    for device in snap.devices.iter() {
                        DeviceItem {
                            device: device.clone(),
                            snapshot: snap.clone(),
                            expanded: expanded,
                        }
                    }
                }
            }
        }
        _ => {
            rsx! {
                document::Link { rel: "stylesheet", href: TREE_CSS }
                div { class: "tree",
                    div { class: "tree-empty", "EMPTY" }
                }
            }
        }
    }
}

#[component]
fn DeviceItem(
    device: DeviceSnapshot,
    snapshot: Snapshot,
    mut expanded: Signal<HashSet<String>>,
) -> Element {
    let device_key = format!("device-{:?}", device.id);
    let is_expanded = expanded.read().contains(&device_key);

    let floe_name = snapshot
        .floes
        .iter()
        .find(|f| f.id == device.owner)
        .map(|f| format!("#{:?}", f.fref.0))
        .unwrap_or_else(|| format!("{:?}", device.owner));

    let group_names: Vec<String> = snapshot
        .groups
        .iter()
        .filter(|g| g.devices.contains(&device.id))
        .map(|g| g.name.clone())
        .collect();

    rsx! {
        div { class: "tree-device",
            div {
                class: "tree-header",
                onclick: move |_| {
                    let mut exp = expanded.write();
                    if exp.contains(&device_key) {
                        exp.remove(&device_key);
                    } else {
                        exp.insert(device_key.clone());
                    }
                },
                span { class: "tree-icon", if is_expanded { "▼" } else { "▶" } }
                span { "{device.name} ({device.id})" }
                span { class: "tree-badge", "Floe: {floe_name}" }
                for group_name in group_names.iter() {
                    span { class: "tree-badge", "Group: {group_name}" }
                }
            }
            if is_expanded {
                if device.entities.is_empty() {
                    div { class: "tree-empty", "EMPTY" }
                } else {
                    for (idx, entity) in device.entities.iter().enumerate() {
                        EntityItem {
                            entity: entity.clone(),
                            device_id: device.id,
                            entity_index: idx,
                            expanded: expanded,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EntityItem(
    entity: EntitySnapshot,
    device_id: DeviceID,
    entity_index: usize,
    mut expanded: Signal<HashSet<String>>,
) -> Element {
    let entity_key = format!("device-{:?}-entity-{}", device_id, entity_index);
    let is_expanded = expanded.read().contains(&entity_key);

    rsx! {
        div { class: "tree-entity",
            div {
                class: "tree-header",
                onclick: move |_| {
                    let mut exp = expanded.write();
                    if exp.contains(&entity_key) {
                        exp.remove(&entity_key);
                    } else {
                        exp.insert(entity_key.clone());
                    }
                },
                // span { class: "tree-icon", if is_expanded { "▼" } else { "▶" } }
                span { "{entity.name}" }
            }
            if is_expanded {
                if entity.components.is_empty() {
                    div { class: "tree-empty", "EMPTY" }
                } else {
                    for component in entity.components.iter() {
                        ComponentItem {
                            component: component.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComponentItem(component: Component) -> Element {
    rsx! {
        div { class: "tree-component",
            "{component:?}"
        }
    }
}
