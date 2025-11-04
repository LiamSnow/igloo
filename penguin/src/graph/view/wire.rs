use euclid::Box2D;
use igloo_interface::{
    PenguinPinType,
    graph::{PenguinWire, PenguinWireID},
};
use wasm_bindgen::JsValue;
use web_sys::{Element, HtmlElement};

use crate::{
    app::event::{EventTarget, ListenerBuilder, Listeners, document},
    viewport::{ClientBox, ClientToWorld, WorldPoint},
};

#[derive(Debug)]
pub struct WebWire {
    pub inner: PenguinWire,

    from_hitbox: HtmlElement,
    to_hitbox: HtmlElement,

    svg: Element,
    path: Element,

    from_pos: (f64, f64),
    to_pos: (f64, f64),

    listeners: Listeners,
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
    let document = document();

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
        id: PenguinWireID,
        inner: PenguinWire,
        from_hitbox: HtmlElement,
        to_hitbox: HtmlElement,
    ) -> Result<Self, JsValue> {
        let (svg, path) = make_els(parent, inner.r#type)?;

        let listeners = ListenerBuilder::new(&path, EventTarget::Wire(id))
            .add_mousedown()?
            .add_contextmenu()?
            .build();

        Ok(Self {
            inner,
            from_hitbox,
            to_hitbox,
            svg,
            path,
            from_pos: (0., 0.),
            to_pos: (0., 0.),
            listeners,
        })
    }

    // TODO FIXME calculation is slightly off.
    // Change function to take in client -> world transform
    // and use the from_hitbox bounding client rect
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

    pub fn select(&self, selected: bool) -> Result<(), JsValue> {
        if selected {
            self.path.set_attribute("class", "selected")
        } else {
            self.path.set_attribute("class", "")
        }
    }

    pub fn intersects(&self, cbox: &ClientBox, ctw: &ClientToWorld) -> bool {
        let wbox = ctw.outer_transformed_box(&cbox.to_f64());

        let (from_x, from_y) = self.from_pos;
        let (to_x, to_y) = self.to_pos;
        let offset = (to_x - from_x).abs() * 0.5;
        let cx1 = from_x + offset;
        let cx2 = to_x - offset;

        let min_x = from_x.min(cx1).min(cx2).min(to_x);
        let max_x = from_x.max(cx1).max(cx2).max(to_x);
        let min_y = from_y.min(to_y);
        let max_y = from_y.max(to_y);

        let wire_bbox = Box2D::new(WorldPoint::new(min_x, min_y), WorldPoint::new(max_x, max_y));

        if !wire_bbox.intersects(&wbox) {
            return false;
        }

        let dx = to_x - from_x;
        let dy = to_y - from_y;
        let length = (dx * dx + dy * dy).sqrt();
        let samples = ((length / 10.0).ceil() as usize).clamp(10, 500);

        for i in 0..=samples {
            let t = i as f64 / samples as f64;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let t2 = t * t;
            let t3 = t2 * t;

            let x = mt3 * from_x + 3.0 * mt2 * t * cx1 + 3.0 * mt * t2 * cx2 + t3 * to_x;
            let y = mt3 * from_y + 3.0 * mt2 * t * from_y + 3.0 * mt * t2 * to_y + t3 * to_y;

            let point = WorldPoint::new(x, y);
            if wbox.contains(point) {
                return true;
            }
        }

        false
    }

    pub fn inner(&self) -> &PenguinWire {
        &self.inner
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
        start_hitbox: &HtmlElement,
        start_node_pos: WorldPoint,
        r#type: PenguinPinType,
        is_output: bool,
    ) -> Result<(), JsValue> {
        self.path.set_attribute("stroke", r#type.stroke())?;
        self.path
            .set_attribute("stroke-width", &r#type.stroke_width().to_string())?;

        self.start_pos.0 = start_node_pos.x
            + start_hitbox.offset_left() as f64
            + start_hitbox.offset_width() as f64 / 2.0;
        self.start_pos.1 = start_node_pos.y
            + start_hitbox.offset_top() as f64
            + start_hitbox.offset_height() as f64 / 2.0;

        self.is_output = is_output;

        self.svg.remove_attribute("style")?;

        Ok(())
    }

    pub fn update(&self, mouse_pos: WorldPoint) -> Result<(), JsValue> {
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
