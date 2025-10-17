use dioxus::{prelude::*, router::RouterConfig};

mod dash;
mod header;
mod mouse;
mod penguin;
mod settings;
mod sidebar;
mod tree;
mod ws;

use dash::{Dash, DashDefault};
use header::Header;
use penguin::Penguin;
use settings::Settings;
use tree::Tree;

use crate::ws::CURRENT_ROUTE;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Header)]
    #[redirect("/", || Route::DashDefault { })]

    #[route("/dash")]
    DashDefault { },
    #[route("/dash/:id")]
    Dash { id: String },

    #[route("/tree")]
    Tree { },

    #[route("/penguin")]
    Penguin { },
    // TODO nest penguin pages

    #[route("/settings")]
    Settings { },
    // TODO nest settings pages https://dioxuslabs.com/learn/0.7/essentials/router/nested-routes
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

fn main() {
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_hook(|| {
        mouse::init();
    });

    // note connection logic is in Header to access cur route
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {
            config: || RouterConfig::default()
                .on_update(|state| {
                    *CURRENT_ROUTE.write() = state.current();
                    ws::send_cur_page();
                    None
                })
        }
    }
}
