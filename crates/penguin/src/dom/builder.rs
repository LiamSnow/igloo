use std::marker::PhantomData;

use crate::dom::{
    Button, Circle, Div, Input, Path, Pattern, Polygon, Rect, Svg, TextArea, events::EventTarget,
};

use super::{js, node::DomNode};
use web_sys::Element;

pub struct DomBuilder<T = ()> {
    pub(super) node: DomNode<T>,
    _marker: PhantomData<T>,
}

impl<T> DomBuilder<T> {
    pub(super) fn new(element: Element) -> Self {
        Self {
            node: DomNode::new(element),
            _marker: PhantomData,
        }
    }

    pub fn event_target(mut self, target: EventTarget) -> Self {
        self.node.event_target(target);
        self
    }

    pub fn remove_on_drop(mut self) -> Self {
        self.node.remove_on_drop();
        self
    }

    pub fn build(self) -> DomNode<T> {
        self.node
    }

    pub fn class(self, name: &str) -> Self {
        self.node.set_class(name);
        self
    }

    pub fn id(self, id: &str) -> Self {
        self.node.set_id(id);
        self
    }

    pub fn attr(self, key: &str, value: &str) -> Self {
        self.node.set_attr(key, value);
        self
    }

    #[allow(dead_code)]
    pub fn html(self, html: &str) -> Self {
        self.node.set_html(html);
        self
    }

    pub fn text(self, text: &str) -> Self {
        self.node.set_text(text);
        self
    }

    #[allow(dead_code)]
    pub fn show(self) -> Self {
        self.node.show();
        self
    }

    pub fn hide(self) -> Self {
        self.node.hide();
        self
    }

    pub fn style(self, key: &str, value: &str) -> Self {
        self.node.set_style(key, value);
        self
    }

    #[allow(dead_code)]
    pub fn left(self, value: f64) -> Self {
        self.node.set_left(value);
        self
    }

    #[allow(dead_code)]
    pub fn top(self, value: f64) -> Self {
        self.node.set_top(value);
        self
    }

    #[allow(dead_code)]
    pub fn right(self, value: f64) -> Self {
        self.node.set_right(value);
        self
    }

    #[allow(dead_code)]
    pub fn bottom(self, value: f64) -> Self {
        self.node.set_bottom(value);
        self
    }

    #[allow(dead_code)]
    pub fn width(self, value: f64) -> Self {
        self.node.set_width(value);
        self
    }

    #[allow(dead_code)]
    pub fn height(self, value: f64) -> Self {
        self.node.set_height(value);
        self
    }

    #[allow(dead_code)]
    pub fn size(self, width: f64, height: f64) -> Self {
        self.node.set_size(width, height);
        self
    }

    #[allow(dead_code)]
    pub fn translate(self, x: f64, y: f64) -> Self {
        self.node.translate(x, y);
        self
    }

    #[allow(dead_code)]
    pub fn translate_scale(self, x: f64, y: f64, scale: f64) -> Self {
        self.node.translate_scale(x, y, scale);
        self
    }

    pub fn mount<P>(self, parent: &DomNode<P>) -> DomNode<T> {
        js::append_child(&parent.element, &self.node.element);
        self.node
    }
}

impl DomBuilder<Input> {
    pub fn value(self, value: &str) -> Self {
        self.node.set_value(value);
        self
    }

    #[allow(dead_code)]
    pub fn checked(self, checked: bool) -> Self {
        self.node.set_checked(checked);
        self
    }

    pub fn placeholder(self, text: &str) -> Self {
        self.node.set_placeholder(text);
        self
    }

    pub fn type_attr(self, t: &str) -> Self {
        self.node.set_type(t);
        self
    }
}

impl DomBuilder<TextArea> {
    pub fn value(self, value: &str) -> Self {
        js::set_value(&self.node.element, value);
        self
    }

    #[allow(dead_code)]
    pub fn placeholder(self, text: &str) -> Self {
        js::set_placeholder(&self.node.element, text);
        self
    }
}

impl DomBuilder<Button> {
    #[allow(dead_code)]
    pub fn tab_index(self, idx: i32) -> Self {
        self.node.set_tab_index(idx);
        self
    }
}

impl DomBuilder<Div> {
    pub fn tab_index(self, idx: i32) -> Self {
        self.node.set_tab_index(idx);
        self
    }
}

impl DomBuilder<Svg> {
    pub fn viewbox(self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.node.set_viewbox(x, y, width, height);
        self
    }
}

impl DomBuilder<Path> {
    #[allow(dead_code)]
    pub fn path_d(self, d: &str) -> Self {
        js::set_path_d(&self.node.element, d);
        self
    }

    pub fn stroke(self, color: &str) -> Self {
        self.node.set_stroke(color);
        self
    }

    pub fn stroke_width(self, width: f64) -> Self {
        self.node.set_stroke_width(width);
        self
    }

    pub fn fill(self, color: &str) -> Self {
        self.node.set_fill(color);
        self
    }

    #[allow(dead_code)]
    pub fn stroke_dasharray(self, pattern: &str) -> Self {
        self.node.set_stroke_dasharray(pattern);
        self
    }
}

impl DomBuilder<Polygon> {
    pub fn stroke(self, color: &str) -> Self {
        self.node.set_stroke(color);
        self
    }

    pub fn stroke_width(self, width: f64) -> Self {
        self.node.set_stroke_width(width);
        self
    }

    pub fn fill(self, color: &str) -> Self {
        self.node.set_fill(color);
        self
    }

    pub fn points(self, points: &str) -> Self {
        self.node.set_points(points);
        self
    }
}

impl DomBuilder<Circle> {
    pub fn cx(self, value: f64) -> Self {
        self.node.set_cx(value);
        self
    }

    pub fn cy(self, value: f64) -> Self {
        self.node.set_cy(value);
        self
    }

    pub fn r(self, value: f64) -> Self {
        self.node.set_r(value);
        self
    }

    pub fn fill(self, color: &str) -> Self {
        self.node.set_fill(color);
        self
    }

    #[allow(dead_code)]
    pub fn stroke(self, color: &str) -> Self {
        self.node.set_stroke(color);
        self
    }
}

impl DomBuilder<Rect> {
    pub fn x(self, value: f64) -> Self {
        self.node.set_x(value);
        self
    }

    pub fn y(self, value: f64) -> Self {
        self.node.set_y(value);
        self
    }

    #[allow(dead_code)]
    pub fn svg_width(self, value: f64) -> Self {
        self.node.set_svg_width(value);
        self
    }

    #[allow(dead_code)]
    pub fn svg_height(self, value: f64) -> Self {
        self.node.set_svg_height(value);
        self
    }

    pub fn fill(self, color: &str) -> Self {
        self.node.set_fill(color);
        self
    }
}

impl DomBuilder<Pattern> {
    pub fn x(self, value: f64) -> Self {
        self.node.set_x(value);
        self
    }

    pub fn y(self, value: f64) -> Self {
        self.node.set_y(value);
        self
    }

    pub fn pattern_units(self, units: &str) -> Self {
        self.node.set_pattern_units(units);
        self
    }

    #[allow(dead_code)]
    pub fn svg_width(self, value: f64) -> Self {
        self.node.set_svg_width(value);
        self
    }

    #[allow(dead_code)]
    pub fn svg_height(self, value: f64) -> Self {
        self.node.set_svg_height(value);
        self
    }
}
