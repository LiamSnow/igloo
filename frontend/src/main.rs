use dioxus::{prelude::*, router::RouterConfig};

mod dash;
mod header;
mod mouse;
mod penguin;
mod settings;
mod ws;

use dash::Dash;
use header::Header;
use penguin::Penguin;
use settings::Settings;

use crate::ws::CURRENT_DASHBOARD_ID;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Header)]
    #[redirect("/", || Route::Dash { id: 0 })]
    #[route("/dash/:id")]
    Dash { id: u16 },
    #[route("/penguin")]
    Penguin { },
    // TODO nest penguin pages
    #[route("/settings")]
    Settings { },
    // TODO nest settings pages
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
                    set_current_dash_id(&state.current());
                    ws::send_dash_id();
                    None
                })
        }
    }
}

pub fn set_current_dash_id(route: &Route) {
    let id = match route {
        Route::Dash { id } => *id,
        _ => u16::MAX,
    };
    *CURRENT_DASHBOARD_ID.write() = id;
}
