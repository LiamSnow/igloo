use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Document, Element, MouseEvent};

use crate::{app::APP, ffi};

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
    settings: GridSettings,
    pub grid_svg: Element,
    pattern_el: Element,
    rect_el: Element,
    closures: Vec<Closure<dyn FnMut(MouseEvent)>>,
}

impl Grid {
    pub fn new(
        penguin_el: &Element,
        grid_svg: Element,
        settings: GridSettings,
    ) -> Result<Self, JsValue> {
        let document = ffi::document();

        let defs = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "defs")?;
        grid_svg.append_child(&defs)?;

        let pattern_el =
            document.create_element_ns(Some("http://www.w3.org/2000/svg"), "pattern")?;
        pattern_el.set_id("penguin-dot-grid");
        pattern_el.set_attribute("x", "0")?;
        pattern_el.set_attribute("y", "0")?;
        pattern_el.set_attribute("width", &settings.size.to_string())?;
        pattern_el.set_attribute("height", &settings.size.to_string())?;
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
        if !settings.enabled {
            rect_el.set_attribute("style", "display: none;")?;
        }
        grid_svg.append_child(&rect_el)?;

        Ok(Self {
            closures: make_toolbar(&document, penguin_el, &settings)?,
            settings,
            grid_svg,
            pattern_el,
            rect_el,
        })
    }

    fn set_enabled(&mut self, enabled: bool) -> Result<(), JsValue> {
        self.settings.enabled = enabled;
        if self.settings.enabled {
            self.rect_el.remove_attribute("style")
        } else {
            self.rect_el.set_attribute("style", "display: none;")
        }
    }

    fn set_size(&mut self, size: f64) -> Result<(), JsValue> {
        let s = size.to_string();
        self.settings.size = size;
        self.pattern_el.set_attribute("width", &s)?;
        self.pattern_el.set_attribute("height", &s)
    }

    pub fn settings(&self) -> &GridSettings {
        &self.settings
    }
}

fn make_toolbar(
    document: &Document,
    penguin_el: &Element,
    settings: &GridSettings,
) -> Result<Vec<Closure<dyn FnMut(MouseEvent)>>, JsValue> {
    let toolbar = document.create_element("div")?;
    toolbar.set_class_name("penguin-grid-settings-toolbar");
    penguin_el.append_child(&toolbar)?;

    let mut closures = Vec::with_capacity(3);

    // enabled
    let button = document.create_element("button")?;
    button.set_class_name(if settings.enabled {
        "penguin-grid-setting-button active"
    } else {
        "penguin-grid-setting-button"
    });
    button.set_inner_html("#");
    toolbar.append_child(&button)?;
    // TODO 1: enabled button
    let onclick = Closure::wrap(Box::new(move |e: MouseEvent| {
        if e.button() != 0 {
            return;
        }

        e.prevent_default();
        e.stop_propagation();

        APP.with(|app| {
            let mut b = app.borrow_mut();
            let Some(app) = b.as_mut() else {
                return;
            };

            let grid = &mut app.viewport.grid;
            let new = !grid.settings.enabled;
            grid.set_enabled(new);

            if let Some(target) = e.current_target()
                && let Ok(button) = target.dyn_into::<web_sys::HtmlElement>()
            {
                if new {
                    button.set_class_name("penguin-grid-setting-button active");
                } else {
                    button.set_class_name("penguin-grid-setting-button");
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", onclick.as_ref().unchecked_ref())?;
    closures.push(onclick);

    // snap
    let button = document.create_element("button")?;
    button.set_class_name(if settings.snap {
        "penguin-grid-setting-button active"
    } else {
        "penguin-grid-setting-button"
    });
    button.set_inner_html("S");
    toolbar.append_child(&button)?;
    let onclick = Closure::wrap(Box::new(move |e: MouseEvent| {
        if e.button() != 0 {
            return;
        }

        e.prevent_default();
        e.stop_propagation();

        APP.with(|app| {
            let mut b = app.borrow_mut();
            let Some(app) = b.as_mut() else {
                return;
            };

            let grid = &mut app.viewport.grid;
            let new = !grid.settings.snap;
            grid.settings.snap = new;

            if let Some(target) = e.current_target()
                && let Ok(button) = target.dyn_into::<web_sys::HtmlElement>()
            {
                if new {
                    button.set_class_name("penguin-grid-setting-button active");
                } else {
                    button.set_class_name("penguin-grid-setting-button");
                }
            }
        });
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", onclick.as_ref().unchecked_ref())?;
    closures.push(onclick);

    // size
    let button = document.create_element("button")?;
    button.set_class_name("penguin-grid-setting-button");
    button.set_inner_html(&(settings.size as u8 / 10).to_string());
    toolbar.append_child(&button)?;
    let onclick = Closure::wrap(Box::new(move |e: MouseEvent| {
        if e.button() != 0 {
            return;
        }

        e.prevent_default();
        e.stop_propagation();

        APP.with(|app| {
            let mut b = app.borrow_mut();
            let Some(app) = b.as_mut() else {
                return;
            };

            let grid = &mut app.viewport.grid;
            let new = if grid.settings.size >= 40. {
                10.
            } else {
                grid.settings.size + 10.
            };
            grid.set_size(new);

            if let Some(target) = e.current_target()
                && let Ok(button) = target.dyn_into::<web_sys::HtmlElement>()
            {
                button.set_inner_html(&(new as u8 / 10).to_string());
            }
        });
    }) as Box<dyn FnMut(_)>);
    button.add_event_listener_with_callback("click", onclick.as_ref().unchecked_ref())?;
    closures.push(onclick);

    Ok(closures)
}
