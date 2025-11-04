use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::app::event::document;

#[derive(Clone, Debug, PartialEq)]
pub struct GridSettings {
    pub enabled: bool,
    pub snap: bool,
    pub size: f64,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap: true,
            size: 20.0,
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    pub grid_svg: Element,
    pattern_el: Element,
    rect_el: Element,
}

impl Grid {
    pub fn new(grid_svg: Element) -> Result<Self, JsValue> {
        let document = document();

        let defs = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "defs")?;
        grid_svg.append_child(&defs)?;

        let pattern_el =
            document.create_element_ns(Some("http://www.w3.org/2000/svg"), "pattern")?;
        pattern_el.set_id("penguin-dot-grid");
        pattern_el.set_attribute("x", "0")?;
        pattern_el.set_attribute("y", "0")?;
        pattern_el.set_attribute("patternUnits", "userSpaceOnUse")?;

        let circle = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "circle")?;
        circle.set_attribute("cx", "0")?;
        circle.set_attribute("cy", "0")?;
        circle.set_attribute("r", "1.5")?;
        circle.set_attribute("fill", "rgba(255,255,255,0.15)")?;
        pattern_el.append_child(&circle)?;

        defs.append_child(&pattern_el)?;

        let rect_el = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "rect")?;
        rect_el.set_id("grid-background");
        rect_el.set_attribute("x", "-10000")?;
        rect_el.set_attribute("y", "-10000")?;
        rect_el.set_attribute("width", "20000")?;
        rect_el.set_attribute("height", "20000")?;
        rect_el.set_attribute("fill", "url(#penguin-dot-grid)")?;
        grid_svg.append_child(&rect_el)?;

        Ok(Self {
            grid_svg,
            pattern_el,
            rect_el,
        })
    }

    pub fn update_grid_settings(&self, gs: &GridSettings) -> Result<(), JsValue> {
        if gs.enabled {
            self.rect_el.remove_attribute("style")?;
        } else {
            self.rect_el.set_attribute("style", "display: none;")?;
        }

        let s = gs.size.to_string();
        self.pattern_el.set_attribute("width", &s)?;
        self.pattern_el.set_attribute("height", &s)
    }
}
