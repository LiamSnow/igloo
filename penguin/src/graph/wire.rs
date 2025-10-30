use crate::ffi;

use super::*;
use igloo_interface::{PenguinPinType, graph::PenguinWire};
use web_sys::Element;

#[derive(Debug)]
pub struct WebWire {
    pub inner: PenguinWire,

    from_hitbox: HtmlElement,
    to_hitbox: HtmlElement,

    svg: Element,
    path: Element,

    from_pos: (f64, f64),
    to_pos: (f64, f64),
}

#[derive(Debug)]
pub struct WebTempWire {
    r#type: PenguinPinType,
    start_pos: (f64, f64),
    is_output: bool,
    svg: Element,
    path: Element,
}

fn make_els(parent: &Element, r#type: PenguinPinType) -> Result<(Element, Element), JsValue> {
    let document = ffi::document();

    let svg = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")?;
    svg.set_attribute("class", "penguin-wire")?;
    parent.append_child(&svg)?;

    let path = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "path")?;
    path.set_attribute("stroke", r#type.stroke())?;
    path.set_attribute("stroke-width", &r#type.stroke_width().to_string())?;
    path.set_attribute("fill", "none")?;
    svg.append_child(&path)?;

    Ok((svg, path))
}

impl WebWire {
    pub fn new(
        parent: &Element,
        inner: PenguinWire,
        from_hitbox: HtmlElement,
        to_hitbox: HtmlElement,
    ) -> Result<Self, JsValue> {
        let (svg, path) = make_els(parent, inner.r#type)?;

        Ok(Self {
            inner,
            from_hitbox,
            to_hitbox,
            svg,
            path,
            from_pos: (0., 0.),
            to_pos: (0., 0.),
        })
    }

    pub fn redraw_from(&mut self, from_node_pos: WorldPoint) -> Result<(), JsValue> {
        self.from_pos.0 = from_node_pos.x
            + self.from_hitbox.offset_left() as f64
            + self.from_hitbox.offset_width() as f64 / 2.0;
        self.from_pos.1 = from_node_pos.y
            + self.from_hitbox.offset_top() as f64
            + self.from_hitbox.offset_height() as f64 / 2.0;

        self.update_path()
    }

    pub fn redraw_to(&mut self, to_node_pos: WorldPoint) -> Result<(), JsValue> {
        self.to_pos.0 = to_node_pos.x
            + self.to_hitbox.offset_left() as f64
            + self.to_hitbox.offset_width() as f64 / 2.0;
        self.to_pos.1 = to_node_pos.y
            + self.to_hitbox.offset_top() as f64
            + self.to_hitbox.offset_height() as f64 / 2.0;

        self.update_path()
    }

    fn update_path(&self) -> Result<(), JsValue> {
        draw_bezier_path(&self.path, self.from_pos, self.to_pos)
    }
}

impl Drop for WebWire {
    fn drop(&mut self) {
        self.svg.remove();
    }
}

impl WebTempWire {
    pub fn new(parent: &Element) -> Result<Self, JsValue> {
        let r#type = PenguinPinType::Flow;

        let (svg, path) = make_els(parent, r#type)?;
        svg.set_id("penguin-temp-wire");
        path.set_attribute("stroke-dasharray", "5 5")?;

        let me = Self {
            r#type,
            start_pos: (0., 0.),
            is_output: false,
            svg,
            path,
        };

        me.hide()?;

        Ok(me)
    }

    pub fn show(
        &mut self,
        start_el: &HtmlElement,
        start_node_pos: WorldPoint,
        r#type: PenguinPinType,
        is_output: bool,
    ) -> Result<(), JsValue> {
        self.path.set_attribute("stroke", r#type.stroke())?;
        self.path
            .set_attribute("stroke-width", &r#type.stroke_width().to_string())?;

        self.start_pos.0 =
            start_node_pos.x + start_el.offset_left() as f64 + start_el.offset_width() as f64 / 2.0;
        self.start_pos.1 =
            start_node_pos.y + start_el.offset_top() as f64 + start_el.offset_height() as f64 / 2.0;

        self.is_output = is_output;

        self.svg.remove_attribute("style")?;

        Ok(())
    }

    pub fn update(&mut self, mouse_pos: WorldPoint) -> Result<(), JsValue> {
        let (from, to) = if self.is_output {
            (self.start_pos, (mouse_pos.x, mouse_pos.y))
        } else {
            ((mouse_pos.x, mouse_pos.y), self.start_pos)
        };

        draw_bezier_path(&self.path, from, to)
    }

    pub fn hide(&self) -> Result<(), JsValue> {
        self.svg.set_attribute("style", "display: none;")
    }
}

fn draw_bezier_path(path_el: &Element, from: (f64, f64), to: (f64, f64)) -> Result<(), JsValue> {
    let (from_x, from_y) = from;
    let (to_x, to_y) = to;

    let offset = (to_x - from_x).abs() * 0.5;
    let cx1 = from_x + offset;
    let cx2 = to_x - offset;

    let path_data = format!(
        "M {} {} C {} {}, {} {}, {} {}",
        from_x, from_y, cx1, from_y, cx2, to_y, to_x, to_y
    );
    path_el.set_attribute("d", &path_data)
}
