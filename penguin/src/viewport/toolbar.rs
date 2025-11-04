use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
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
    el: Element,
    grid_enable_el: Element,
    grid_snap_el: Element,
    grid_size_el: Element,
    listeners: [Listeners; 3],
}

impl Toolbar {
    pub fn new(parent: &Element) -> Result<Self, JsValue> {
        let document = document();

        let el = document.create_element("div")?;
        el.set_class_name("penguin-grid-settings-toolbar");
        parent.append_child(&el)?;

        let grid_enable_el = document.create_element("button")?;
        grid_enable_el.set_inner_html("#");
        el.append_child(&grid_enable_el)?;

        let grid_snap_el = document.create_element("button")?;
        grid_snap_el.set_inner_html("S");
        el.append_child(&grid_snap_el)?;

        let grid_size_el = document.create_element("button")?;
        grid_size_el.set_class_name("penguin-grid-setting-button");
        el.append_child(&grid_size_el)?;

        let listeners = [
            ListenerBuilder::new(
                &grid_enable_el,
                EventTarget::ToolbarButton(ToolbarButton::GridEnable),
            )
            .add_mouseclick()?
            .build(),
            ListenerBuilder::new(
                &grid_snap_el,
                EventTarget::ToolbarButton(ToolbarButton::GridSnap),
            )
            .add_mouseclick()?
            .build(),
            ListenerBuilder::new(
                &grid_size_el,
                EventTarget::ToolbarButton(ToolbarButton::GridSize),
            )
            .add_mouseclick()?
            .build(),
        ];

        Ok(Self {
            el,
            grid_enable_el,
            grid_snap_el,
            grid_size_el,
            listeners,
        })
    }

    pub fn update_grid_settings(&self, settings: &GridSettings) -> Result<(), JsValue> {
        if settings.enabled {
            self.grid_enable_el
                .set_class_name("penguin-grid-setting-button active");
        } else {
            self.grid_enable_el
                .set_class_name("penguin-grid-setting-button");
        }

        if settings.snap {
            self.grid_snap_el
                .set_class_name("penguin-grid-setting-button active");
        } else {
            self.grid_snap_el
                .set_class_name("penguin-grid-setting-button");
        }

        self.grid_size_el
            .set_inner_html(&(settings.size as u8 / 10).to_string());

        Ok(())
    }
}
