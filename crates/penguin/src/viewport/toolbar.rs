use crate::{
    dom::{self, Button, events::EventTarget, node::DomNode},
    viewport::grid::GridSettings,
};

#[derive(Clone, Debug, Copy)]
pub enum ToolbarButton {
    GridEnable,
    GridSnap,
    GridSize,
}

#[derive(Debug)]
pub struct Toolbar {
    grid_enable_el: DomNode<Button>,
    grid_snap_el: DomNode<Button>,
    grid_size_el: DomNode<Button>,
}

impl Toolbar {
    pub fn new<T>(parent: &DomNode<T>) -> Self {
        let el = dom::div()
            .class("penguin-grid-settings-toolbar")
            .mount(parent);

        let grid_enable_el = dom::button()
            .class("penguin-grid-setting-button")
            .text("#")
            .event_target(EventTarget::ToolbarButton(ToolbarButton::GridEnable))
            .listen_click()
            .remove_on_drop()
            .mount(&el);

        let grid_snap_el = dom::button()
            .class("penguin-grid-setting-button")
            .text("S")
            .event_target(EventTarget::ToolbarButton(ToolbarButton::GridSnap))
            .listen_click()
            .remove_on_drop()
            .mount(&el);

        let grid_size_el = dom::button()
            .class("penguin-grid-setting-button")
            .event_target(EventTarget::ToolbarButton(ToolbarButton::GridSize))
            .listen_click()
            .remove_on_drop()
            .mount(&el);

        Self {
            grid_enable_el,
            grid_snap_el,
            grid_size_el,
        }
    }

    pub fn update_grid_settings(&self, settings: &GridSettings) {
        if settings.enabled {
            self.grid_enable_el
                .set_class("penguin-grid-setting-button active");
        } else {
            self.grid_enable_el.set_class("penguin-grid-setting-button");
        }

        if settings.snap {
            self.grid_snap_el
                .set_class("penguin-grid-setting-button active");
        } else {
            self.grid_snap_el.set_class("penguin-grid-setting-button");
        }

        self.grid_size_el
            .set_text(&(settings.size as u8 / 10).to_string());
    }
}
