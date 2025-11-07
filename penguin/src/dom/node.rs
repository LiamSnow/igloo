use crate::dom::events::EventTarget;
use crate::dom::{Button, Circle, Div, Input, Path, Pattern, Polygon, Rect, Svg, TextArea, js};
use crate::viewport::ClientSpace;
use euclid::Box2D;
use std::any::Any;
use std::marker::PhantomData;
use web_sys::{Element, ResizeObserver};

#[derive(Debug)]
pub struct DomNode<T = ()> {
    pub element: Element,
    pub(super) closures: Vec<Box<dyn Any>>,
    pub(super) observer: Option<ResizeObserver>,
    pub(super) event_target: Option<EventTarget>,
    pub remove_on_drop: bool,
    _marker: PhantomData<T>,
}

impl<T> Drop for DomNode<T> {
    fn drop(&mut self) {
        if let Some(observer) = &self.observer {
            observer.disconnect();
        }
        if self.remove_on_drop {
            self.element.remove();
        }
    }
}

impl<T> DomNode<T> {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            closures: Vec::new(),
            observer: None,
            event_target: None,
            remove_on_drop: false,
            _marker: PhantomData,
        }
    }

    pub fn dupe(&self) -> Self {
        Self {
            element: self.element.clone(),
            closures: Vec::new(),
            observer: None,
            event_target: None,
            remove_on_drop: false,
            _marker: PhantomData,
        }
    }

    pub fn remove_on_drop(&mut self) {
        self.remove_on_drop = true;
    }

    pub fn event_target(&mut self, target: EventTarget) {
        self.event_target = Some(target);
    }

    #[allow(dead_code)]
    pub fn element(&self) -> &Element {
        &self.element
    }

    pub(super) fn add_closure(&mut self, closure: Box<dyn Any>) {
        self.closures.push(closure);
    }

    pub(super) fn set_observer(&mut self, observer: ResizeObserver) {
        self.observer = Some(observer);
    }

    pub fn set_attr(&self, key: &str, value: &str) {
        js::set_attr(&self.element, key, value);
    }

    #[allow(dead_code)]
    pub fn remove_attr(&self, key: &str) {
        js::remove_attr(&self.element, key);
    }

    pub fn set_class(&self, name: &str) {
        js::set_class(&self.element, name);
    }

    pub fn set_id(&self, id: &str) {
        js::set_id(&self.element, id);
    }

    pub fn set_html(&self, html: &str) {
        js::set_html(&self.element, html);
    }

    pub fn set_text(&self, text: &str) {
        js::set_text(&self.element, text);
    }

    pub fn show(&self) {
        js::show(&self.element);
    }

    pub fn hide(&self) {
        js::hide(&self.element);
    }

    pub fn set_visible(&self, visible: bool) {
        if visible {
            self.show();
        } else {
            self.hide();
        }
    }

    pub fn set_style(&self, key: &str, value: &str) {
        js::set_style(&self.element, key, value);
    }

    pub fn set_left(&self, value: f64) {
        js::set_left(&self.element, value);
    }

    pub fn set_top(&self, value: f64) {
        js::set_top(&self.element, value);
    }

    pub fn set_right(&self, value: f64) {
        js::set_right(&self.element, value);
    }

    pub fn set_bottom(&self, value: f64) {
        js::set_bottom(&self.element, value);
    }

    pub fn set_width(&self, value: f64) {
        js::set_width(&self.element, value);
    }

    pub fn set_height(&self, value: f64) {
        js::set_height(&self.element, value);
    }

    pub fn set_size(&self, width: f64, height: f64) {
        js::set_size(&self.element, width, height);
    }

    pub fn translate(&self, x: f64, y: f64) {
        js::translate(&self.element, x, y);
    }

    pub fn translate_scale(&self, x: f64, y: f64, scale: f64) {
        js::translate_scale(&self.element, x, y, scale);
    }

    pub fn client_box(&self) -> Box2D<f64, ClientSpace> {
        js::get_client_rect(&self.element)
    }

    #[allow(dead_code)]
    pub fn append_child<C>(&self, child: &DomNode<C>) {
        js::append_child(&self.element, &child.element);
    }

    pub fn remove(&self) {
        js::remove_element(&self.element);
    }
}

impl DomNode<Input> {
    pub fn value(&self) -> String {
        js::get_value(&self.element)
    }

    pub fn set_value(&self, value: &str) {
        js::set_value(&self.element, value);
    }

    #[allow(dead_code)]
    pub fn checked(&self) -> bool {
        js::get_checked(&self.element)
    }

    pub fn set_checked(&self, checked: bool) {
        js::set_checked(&self.element, checked);
    }

    pub fn set_placeholder(&self, text: &str) {
        js::set_placeholder(&self.element, text);
    }

    pub fn set_type(&self, t: &str) {
        js::set_type(&self.element, t);
    }

    pub fn focus(&self) {
        js::focus(&self.element);
    }

    #[allow(dead_code)]
    pub fn blur(&self) {
        js::blur(&self.element);
    }
}

impl DomNode<TextArea> {
    #[allow(dead_code)]
    pub fn value(&self) -> String {
        js::get_value(&self.element)
    }

    pub fn set_value(&self, value: &str) {
        js::set_value(&self.element, value);
    }

    #[allow(dead_code)]
    pub fn focus(&self) {
        js::focus(&self.element);
    }

    #[allow(dead_code)]
    pub fn blur(&self) {
        js::blur(&self.element);
    }
}

impl DomNode<Button> {
    #[allow(dead_code)]
    pub fn focus(&self) {
        js::focus(&self.element);
    }

    #[allow(dead_code)]
    pub fn blur(&self) {
        js::blur(&self.element);
    }

    pub fn set_tab_index(&self, idx: i32) {
        js::set_tab_index(&self.element, idx);
    }
}

impl DomNode<Div> {
    pub fn focus(&self) {
        js::focus(&self.element);
    }

    #[allow(dead_code)]
    pub fn blur(&self) {
        js::blur(&self.element);
    }

    #[allow(dead_code)]
    pub fn offset_width(&self) -> i32 {
        js::get_offset_width(&self.element)
    }

    #[allow(dead_code)]
    pub fn offset_height(&self) -> i32 {
        js::get_offset_height(&self.element)
    }

    pub fn set_tab_index(&self, idx: i32) {
        js::set_tab_index(&self.element, idx);
    }
}

impl DomNode<Svg> {
    pub fn set_viewbox(&self, x: f64, y: f64, width: f64, height: f64) {
        js::set_viewbox(&self.element, x, y, width, height);
    }
}

impl DomNode<Path> {
    #[allow(dead_code)]
    pub fn set_path_d(&self, d: &str) {
        js::set_path_d(&self.element, d);
    }

    pub fn set_path_bezier(
        &self,
        x1: f64,
        y1: f64,
        cx1: f64,
        cy1: f64,
        cx2: f64,
        cy2: f64,
        x2: f64,
        y2: f64,
    ) {
        js::set_path_bezier(&self.element, x1, y1, cx1, cy1, cx2, cy2, x2, y2);
    }

    pub fn set_stroke(&self, color: &str) {
        js::set_stroke(&self.element, color);
    }

    pub fn set_stroke_width(&self, width: f64) {
        js::set_stroke_width(&self.element, width);
    }

    pub fn set_fill(&self, color: &str) {
        js::set_fill(&self.element, color);
    }

    pub fn set_stroke_dasharray(&self, pattern: &str) {
        js::set_stroke_dasharray(&self.element, pattern);
    }
}

impl DomNode<Polygon> {
    pub fn set_points(&self, points: &str) {
        js::set_points(&self.element, points);
    }

    pub fn set_stroke(&self, color: &str) {
        js::set_stroke(&self.element, color);
    }

    pub fn set_stroke_width(&self, width: f64) {
        js::set_stroke_width(&self.element, width);
    }

    pub fn set_fill(&self, color: &str) {
        js::set_fill(&self.element, color);
    }
}

impl DomNode<Circle> {
    pub fn set_cx(&self, value: f64) {
        js::set_cx(&self.element, value);
    }

    pub fn set_cy(&self, value: f64) {
        js::set_cy(&self.element, value);
    }

    pub fn set_r(&self, value: f64) {
        js::set_r(&self.element, value);
    }

    pub fn set_fill(&self, color: &str) {
        js::set_fill(&self.element, color);
    }

    pub fn set_stroke(&self, color: &str) {
        js::set_stroke(&self.element, color);
    }
}

impl DomNode<Rect> {
    pub fn set_x(&self, value: f64) {
        js::set_x(&self.element, value);
    }

    pub fn set_y(&self, value: f64) {
        js::set_y(&self.element, value);
    }

    pub fn set_svg_width(&self, value: f64) {
        js::set_svg_width(&self.element, value);
    }

    pub fn set_svg_height(&self, value: f64) {
        js::set_svg_height(&self.element, value);
    }

    pub fn set_fill(&self, color: &str) {
        js::set_fill(&self.element, color);
    }
}

impl DomNode<Pattern> {
    pub fn set_x(&self, value: f64) {
        js::set_x(&self.element, value);
    }

    pub fn set_y(&self, value: f64) {
        js::set_y(&self.element, value);
    }

    pub fn set_pattern_units(&self, units: &str) {
        js::set_pattern_units(&self.element, units);
    }

    pub fn set_svg_width(&self, value: f64) {
        js::set_svg_width(&self.element, value);
    }

    pub fn set_svg_height(&self, value: f64) {
        js::set_svg_height(&self.element, value);
    }
}
