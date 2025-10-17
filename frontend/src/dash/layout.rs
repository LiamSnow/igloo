use std::collections::HashMap;
use std::sync::Arc;

use dioxus::prelude::*;
use igloo_interface::{
    dash::{HStackElement, VStackElement},
    QueryTarget,
};

use super::DashComponent;

#[derive(PartialEq, Props, Clone)]
pub(crate) struct HStackProps {
    pub element: HStackElement,
    pub targets: Arc<HashMap<String, QueryTarget>>,
}

#[component]
pub(crate) fn HStack(props: HStackProps) -> Element {
    let overflow = if props.element.scroll { "auto" } else { "visible" };
    rsx! {
        div {
            style: "display: flex; flex-direction: row; justify-content: {props.element.justify}; align-items: {props.element.align}; overflow-x: {overflow};",
            {props.element.children.iter().map(|child| rsx! {
                DashComponent {
                    el: child.clone(),
                    targets: props.targets.clone()
                }
            })}
        }
    }
}

#[derive(PartialEq, Props, Clone)]
pub(crate) struct VStackProps {
    pub element: VStackElement,
    pub targets: Arc<HashMap<String, QueryTarget>>,
}

#[component]
pub(crate) fn VStack(props: VStackProps) -> Element {
    let overflow = if props.element.scroll { "auto" } else { "visible" };
    rsx! {
        div {
            style: "display: flex; flex-direction: column; justify-content: {props.element.justify}; align-items: {props.element.align}; overflow-y: {overflow};",
            {props.element.children.iter().map(|child| rsx! {
                DashComponent {
                    el: child.clone(),
                    targets: props.targets.clone()
                }
            })}
        }
    }
}
