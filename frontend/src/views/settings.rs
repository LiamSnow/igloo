use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    rsx! {
        div {
            id: "settings",

            h1 { "This is settings" }
        }
    }
}
