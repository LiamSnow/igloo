use crate::{
    sidebar::{SideBar, SideBarLink},
    ws::{CURRENT_DASHBOARD, DASHBOARDS},
    Route,
};
use dioxus::prelude::*;
use igloo_interface::dash::DashElement;

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
pub fn DashEmpty() -> Element {
    use_effect(|| {
        let nav = navigator();
        if let Some(dashboards) = DASHBOARDS.read().as_ref() {
            for dash in dashboards.iter() {
                if dash.is_default {
                    nav.replace(Route::Dash {
                        id: dash.id.clone(),
                    });
                    break;
                }
            }
        }
    });

    // TODO make into func with other one?
    let links = use_memo(move || {
        let dop = DASHBOARDS.read();
        let Some(dashs) = dop.as_ref() else {
            return vec![];
        };
        let mut v = Vec::with_capacity(dashs.len());
        for dash in dashs.iter() {
            v.push(SideBarLink {
                label: dash.display_name.clone(),
                to: Route::Dash {
                    id: dash.id.clone(),
                },
                active: false,
            });
        }
        v
    });

    rsx! {
        document::Link { rel: "stylesheet", href: DASH_CSS }

        SideBar { links: links() }

        div { class: "dashboard",
            h1 { "No dashboards exist. Try creating one." }
        }
    }
}

#[component]
pub fn Dash(id: String) -> Element {
    let links = use_memo(move || {
        let dop = DASHBOARDS.read();
        let Some(dashs) = dop.as_ref() else {
            return vec![];
        };
        let mut v = Vec::with_capacity(dashs.len());
        for dash in dashs.iter() {
            v.push(SideBarLink {
                label: dash.display_name.clone(),
                to: Route::Dash {
                    id: dash.id.clone(),
                },
                active: false,
            });
        }
        v
    });

    rsx! {
        document::Link { rel: "stylesheet", href: DASH_CSS }

        SideBar { links: links() }

        div { class: "dashboard",
            if let Some(dash) = CURRENT_DASHBOARD.read().cloned() {
                DashComponent {
                    el: dash.child,
                }
            }
        }
    }
}

#[component]
fn DashComponent(el: DashElement) -> Element {
    match el {
        DashElement::HStack(el) => rsx! { HStack { el } },
        DashElement::VStack(el) => rsx! { VStack { el } },
        DashElement::Slider(el) => rsx! { Slider { el } },
        DashElement::ColorPicker(el) => rsx! { ColorPicker { el } },
        DashElement::Switch(el) => rsx! { Switch { el } },
        _ => rsx! { div { "Unsupported element" } },
    }
}
