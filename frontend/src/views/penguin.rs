use dioxus::prelude::*;

#[component]
pub fn Penguin() -> Element {
    rsx! {
        div {
            id: "penguin",

            crate::penguin::Workspace {}
        }
    }
}
