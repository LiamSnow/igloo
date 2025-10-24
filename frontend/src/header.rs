use crate::{
    ws::{self, CURRENT_ROUTE, WS_CONNECTED},
    Route,
};
use dioxus::prelude::*;

const HEADER_CSS: Asset = asset!("/assets/styling/header.css");

#[component]
pub fn Header() -> Element {
    let route = use_route::<Route>();

    use_hook(|| {
        *CURRENT_ROUTE.write() = route;
        ws::connect_websocket();
    });

    rsx! {
        document::Link { rel: "stylesheet", href: HEADER_CSS }

        header {
            div { class: "left",
                h1 {
                    "Igloo"
                }
                span {
                    class: if *WS_CONNECTED.read() { "connected" } else { "" }
                }
            },
            div {
                Link {
                    to: Route::DashEmpty {},
                    "Dash"
                }
                Link {
                    to: Route::Penguin {},
                    "Penguin"
                }
                Link {
                    to: Route::Tree {},
                    "Tree"
                }
                Link {
                    to: Route::Settings {},
                    "Settings"
                }
            }
        }

        main {
            Outlet::<Route> {}
        }
    }
}
