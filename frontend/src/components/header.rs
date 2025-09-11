use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::navbar::{Navbar, NavbarItem};

#[component]
pub fn Header() -> Element {
    rsx! {
        Navbar { class: "navbar",
            NavbarItem {
                index: 2usize,
                class: "navbar-item",
                value: "home".to_string(),
                to: Route::Home {},
                "Home"
            }

            NavbarItem {
                index: 2usize,
                class: "navbar-item",
                value: "penguin".to_string(),
                to: Route::Penguin {},
                "penguin"
            }

            NavbarItem {
                index: 2usize,
                class: "navbar-item",
                value: "settings".to_string(),
                to: Route::Settings {},
                "settings"
            }
        }
        Outlet::<Route> {}
    }
}
