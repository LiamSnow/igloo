use std::collections::HashMap;
use std::sync::Arc;

use crate::ws::CURRENT_DASHBOARD;
use dioxus::prelude::*;
use igloo_interface::{dash::DashElement, QueryTarget};

mod color;
mod layout;
mod slider;
mod switch;

use color::ColorPicker;
use layout::{HStack, VStack};
use slider::Slider;
use switch::Switch;

const DASH_CSS: Asset = asset!("/assets/styling/dash.css");

#[component]
pub fn Dash(id: u16) -> Element {
    match CURRENT_DASHBOARD.read().as_ref() {
        Some(dash) => {
            rsx! {
                document::Link { rel: "stylesheet", href: DASH_CSS }
                div { class: "dashboard",
                    DashComponent {
                        el: dash.child.clone(),
                        targets: Arc::new(dash.targets.clone())
                    }
                }
            }
        }
        None => {
            rsx! {
                document::Link { rel: "stylesheet", href: DASH_CSS }
                div { class: "dashboard" }
            }
        }
    }
}

#[derive(PartialEq, Props, Clone)]
struct DashComponentProps {
    el: DashElement,
    targets: Arc<HashMap<String, QueryTarget>>,
}

#[component]
fn DashComponent(props: DashComponentProps) -> Element {
    match props.el {
        DashElement::HStack(e) => {
            rsx! {
                HStack { element: e, targets: props.targets.clone() }
            }
        }
        DashElement::VStack(e) => {
            rsx! {
                VStack { element: e, targets: props.targets.clone() }
            }
        }
        DashElement::Slider(e) => {
            rsx! {
                Slider { element: e, targets: props.targets.clone() }
            }
        }
        DashElement::ColorPicker(e) => {
            rsx! {
                ColorPicker { element: e, targets: props.targets.clone() }
            }
        }
        DashElement::Switch(e) => {
            rsx! {
                Switch { element: e, targets: props.targets.clone() }
            }
        }
        _ => rsx! { div { "Unsupported element" } },
    }
}
