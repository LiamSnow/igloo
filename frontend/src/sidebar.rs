use dioxus::prelude::*;

use crate::Route;

const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");

#[derive(PartialEq, Clone)]
pub struct SideBarLink {
    pub label: String,
    pub to: Route,
    pub active: bool,
}

#[component]
pub fn SideBar(links: Vec<SideBarLink>) -> Element {
    let mut collapsed = use_signal(|| false);

    rsx! {
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }

        aside {
            class: if collapsed() { "sidebar collapsed" } else { "sidebar" },

            button {
                onclick: move |_| collapsed.set(!collapsed()),
                "Toggle"
            }

            nav { class: "sidebar-nav",
                for link in links {
                    Link {
                        to: link.to,
                        class: if link.active {
                            "sidebar-link active"
                        } else {
                            "sidebar-link"
                        },
                        // TODO icons
                        span { class: "sidebar-label",
                            {link.label}
                        }
                    }
                }
            }
        }
    }
}
