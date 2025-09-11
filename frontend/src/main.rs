use dioxus::prelude::*;

use components::Header;
use views::{Home, Penguin, Settings};

mod components;
mod penguin;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Header)]
        #[route("/")]
        Home {},
        #[route("/settings")]
        Settings {},
        #[route("/penguin")]
        Penguin {},
}

const THEME_CSS: Asset = asset!("./assets/theme.css");
const COMPONENTS_CSS: Asset = asset!("./assets/components.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: THEME_CSS }
        document::Stylesheet { href: COMPONENTS_CSS }

        Router::<Route> {}
    }
}
