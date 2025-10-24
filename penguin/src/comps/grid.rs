use dioxus::prelude::*;

use crate::state::GridSettings;

const GRID_SIZES: [f64; 4] = [10.0, 20.0, 30.0, 40.0];

#[component]
pub fn GridSettingsComponent(grid_settings: Signal<GridSettings>) -> Element {
    let current_size_index = GRID_SIZES
        .iter()
        .position(|&s| s == grid_settings().size)
        .unwrap_or(1);

    rsx! {
        div {
            class: "grid-settings-toolbar",

            button {
                class: "grid-setting-button",
                class: if grid_settings().enabled { "active" },
                onclick: move |_| {
                    let mut settings = grid_settings.write();
                    settings.enabled = !settings.enabled;
                },
                title: if grid_settings().enabled { "Hide Grid" } else { "Show Grid" },
                "#"
            }

            button {
                class: "grid-setting-button",
                class: if grid_settings().snap { "active" },
                onclick: move |_| {
                    let mut settings = grid_settings.write();
                    settings.snap = !settings.snap;
                },
                title: if grid_settings().snap { "Disable Snap" } else { "Enable Snap" },
                "S"
            }

            button {
                class: "grid-setting-button",
                onclick: move |_| {
                    let next_index = (current_size_index + 1) % GRID_SIZES.len();
                    let mut settings = grid_settings.write();
                    settings.size = GRID_SIZES[next_index];
                },
                title: "Grid Size: {grid_settings().size}",
                "{current_size_index + 1}"
            }
        }
    }
}
