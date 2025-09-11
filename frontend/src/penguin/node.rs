use dioxus::prelude::*;

use crate::penguin::{coordinates::WorldPoint, model::registry::NodeDefn};

#[derive(Props, PartialEq, Clone)]
pub struct NodeProps {
    id: u32,
    defn: NodeDefn,
    pos: WorldPoint,
}

#[component]
pub fn NodeComponent(props: NodeProps) -> Element {
    rsx! {
        div {
            style: "width: 100px; height: 100px; border: 1px solid white; border-radius: 10px; margin: 10px;",
            span { "{props.defn.title}" }
        }
    }
}
