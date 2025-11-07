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
    from_hitbox: DomNode<Div>,
    to_hitbox: DomNode<Div>,
    #[allow(dead_code)]
    svg: DomNode<Svg>,
    path: DomNode<Path>,
    from_pos: (f64, f64),
    to_pos: (f64, f64),
}

#[derive(Debug)]
pub struct WebTempWire {
    is_output: bool,
    svg: DomNode<Svg>,
    path: DomNode<Path>,
    start_pos: (f64, f64),
}

fn make<T>(parent: &DomNode<T>, r#type: PenguinPinType) -> (DomNode<Svg>, DomNode<Path>) {
    let svg = dom::svg()
        .attr("class", "penguin-wire")
        .remove_on_drop()
        .mount(parent);

    let path = dom::path()
        .stroke(r#type.stroke())
        .stroke_width(r#type.stroke_width() as f64)
        .fill("none")
        .mount(&svg);

    (svg, path)
}

impl WebWire {
    pub fn new<T>(
        parent: &DomNode<T>,
        id: PenguinWireID,
        inner: PenguinWire,
        from_hitbox: DomNode<Div>,
        to_hitbox: DomNode<Div>,
    ) -> Self {
        let (svg, mut path) = make(parent, inner.r#type);

        path.event_target(EventTarget::Wire(id));
        path.listen_click();
        path.listen_dblclick();
        path.listen_contextmenu();

        Self {
            inner,
            from_hitbox,
            to_hitbox,
            svg,
            path,
            from_pos: (0., 0.),
            to_pos: (0., 0.),
        }
    }

    pub fn redraw_from(&mut self, ctw: &ClientToWorld) {
        let cpos = self.from_hitbox.client_box().center();
        let wpos = ctw.transform_point(cpos.cast());
        self.from_pos = (wpos.x, wpos.y);
        self.update_path();
    }

    pub fn redraw_to(&mut self, ctw: &ClientToWorld) {
        let cpos = self.to_hitbox.client_box().center();
        let wpos = ctw.transform_point(cpos.cast());
        self.to_pos = (wpos.x, wpos.y);
        self.update_path();
    }

    fn update_path(&self) {
        draw_bezier_path(&self.path, self.from_pos, self.to_pos);
    }

    pub fn select(&self, selected: bool) {
        if selected {
            self.path.set_attr("class", "selected");
        } else {
            self.path.set_attr("class", "");
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

impl WebTempWire {
    pub fn new<T>(parent: &DomNode<T>) -> Self {
        let r#type = PenguinPinType::Flow;

        let (svg, path) = make(parent, r#type);
        svg.set_id("penguin-temp-wire");
        svg.hide();
        path.set_stroke_dasharray("5 5");

        Self {
            start_pos: (0., 0.),
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
        let wpos = ctw.transform_point(cpos.cast());
        self.start_pos = (wpos.x, wpos.y);

        self.is_output = is_output;
        self.svg.show();
    }

    pub fn update(&self, mouse_pos: WorldPoint) {
        let (from, to) = if self.is_output {
            (self.start_pos, (mouse_pos.x, mouse_pos.y))
        } else {
            ((mouse_pos.x, mouse_pos.y), self.start_pos)
        };

        draw_bezier_path(&self.path, from, to);
    }

    pub fn hide(&self) {
        self.svg.hide();
    }
}

fn draw_bezier_path(path_el: &DomNode<Path>, from: (f64, f64), to: (f64, f64)) {
    let (from_x, from_y) = from;
    let (to_x, to_y) = to;

    let offset = (to_x - from_x).abs() * 0.5;
    let cx1 = from_x + offset;
    let cx2 = to_x - offset;

    path_el.set_path_bezier(from_x, from_y, cx1, from_y, cx2, to_y, to_x, to_y);
}
