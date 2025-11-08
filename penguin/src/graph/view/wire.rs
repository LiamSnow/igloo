use crate::{
    dom::{self, Div, Path, Svg, events::EventTarget, node::DomNode},
    viewport::{ClientBox, ClientToWorld, WorldPoint},
};
use euclid::Box2D;
use igloo_interface::penguin::{
    PenguinPinType,
    graph::{PenguinWire, PenguinWireID},
};

#[derive(Debug)]
pub struct WebWire {
    pub inner: PenguinWire,
    pub from: WorldPoint,
    pub to: WorldPoint,
    #[allow(dead_code)]
    svg: DomNode<Svg>,
    path: DomNode<Path>,
    border_path: DomNode<Path>,
}

#[derive(Debug)]
pub struct WebTempWire {
    is_output: bool,
    svg: DomNode<Svg>,
    path: DomNode<Path>,
    start_pos: WorldPoint,
}

impl WebWire {
    pub fn new<T>(parent: &DomNode<T>, id: PenguinWireID, inner: PenguinWire) -> Self {
        let svg = dom::svg()
            .attr("class", "penguin-wire")
            .remove_on_drop()
            .mount(parent);

        let border_path = dom::path()
            .attr("class", "penguin-wire-border")
            .stroke("transparent")
            .stroke_width((inner.r#type.stroke_width() + 4) as f64)
            .fill("none")
            .event_target(EventTarget::Wire(id))
            .listen_click()
            .listen_dblclick()
            .listen_contextmenu()
            .mount(&svg);

        let path = dom::path()
            .attr("class", "penguin-wire-path")
            .stroke(inner.r#type.stroke())
            .stroke_width(inner.r#type.stroke_width() as f64)
            .fill("none")
            .mount(&svg);

        Self {
            inner,
            svg,
            path,
            border_path,
            from: WorldPoint::default(),
            to: WorldPoint::default(),
        }
    }

    pub fn redraw(&self) {
        dom::js::redraw_wire(
            &self.path.element,
            Some(&self.border_path.element),
            self.from.x,
            self.from.y,
            self.to.x,
            self.to.y,
        );
    }

    pub fn select(&self, selected: bool) {
        if selected {
            self.border_path.set_stroke("#2196F3");
        } else {
            self.border_path.set_stroke("transparent");
        }
    }

    fn bezier_control_points(&self) -> (WorldPoint, WorldPoint) {
        let width = self.to.x - self.from.x;
        let height = self.to.y - self.from.y;
        let offset = width.abs().max(height.abs()) * 0.5;

        let cx1 = WorldPoint::new(self.from.x + offset, self.from.y);
        let cx2 = WorldPoint::new(self.to.x - offset, self.to.y);
        (cx1, cx2)
    }

    pub fn intersects(&self, cbox: &ClientBox, ctw: &ClientToWorld) -> bool {
        let wbox = ctw.outer_transformed_box(&cbox.to_f64());

        let (cx1, cx2) = self.bezier_control_points();
        let wire_bbox = Box2D::from_points([self.from, self.to, cx1, cx2]);

        // no overlap
        if !wire_bbox.intersects(&wbox) {
            return false;
        }

        // selection fully contains wire
        if wbox.contains_box(&wire_bbox) {
            return true;
        }

        let dx = self.to.x - self.from.x;
        let dy = self.to.y - self.from.y;
        let length = (dx * dx + dy * dy).sqrt();
        let samples = ((length / 10.0).ceil() as usize).clamp(10, 500);

        (0..=samples).any(|i| {
            let t = i as f64 / samples as f64;
            let point = sample_cubic_bezier(t, self.from, cx1, cx2, self.to);
            wbox.contains(point)
        })
    }

    pub fn inner(&self) -> &PenguinWire {
        &self.inner
    }
}

impl WebTempWire {
    pub fn new<T>(parent: &DomNode<T>) -> Self {
        let r#type = PenguinPinType::Flow;

        let svg = dom::svg()
            .id("penguin-temp-wire")
            .attr("class", "penguin-wire")
            .hide()
            .mount(parent);

        let path = dom::path()
            .stroke(r#type.stroke())
            .stroke_width(r#type.stroke_width() as f64)
            .fill("none")
            .stroke_dasharray("5 5")
            .mount(&svg);

        Self {
            start_pos: WorldPoint::default(),
            is_output: false,
            svg,
            path,
        }
    }

    pub fn show(
        &mut self,
        start_hitbox: &DomNode<Div>,
        r#type: PenguinPinType,
        is_output: bool,
        ctw: &ClientToWorld,
    ) {
        self.path.set_stroke(r#type.stroke());
        self.path.set_stroke_width(r#type.stroke_width() as f64);

        let cpos = start_hitbox.client_box().center();
        self.start_pos = ctw.transform_point(cpos.cast());

        self.is_output = is_output;
        self.svg.show();
    }

    pub fn redraw(&self, mouse_pos: WorldPoint) {
        let (from, to) = if self.is_output {
            (self.start_pos, mouse_pos)
        } else {
            (mouse_pos, self.start_pos)
        };

        dom::js::redraw_wire(&self.path.element, None, from.x, from.y, to.x, to.y);
    }

    pub fn hide(&self) {
        self.svg.hide();
    }
}

fn sample_cubic_bezier(
    t: f64,
    p0: WorldPoint,
    p1: WorldPoint,
    p2: WorldPoint,
    p3: WorldPoint,
) -> WorldPoint {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;

    WorldPoint::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}
