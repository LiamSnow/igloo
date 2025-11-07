pub mod builder;
pub mod events;
pub mod js;
pub mod node;

use builder::DomBuilder;
use web_sys::Element;

#[derive(Debug, Clone)]
pub struct Div;
#[derive(Debug, Clone)]
pub struct Button;
#[derive(Debug, Clone)]
pub struct Input;
#[derive(Debug, Clone)]
pub struct TextArea;
#[derive(Debug, Clone)]
pub struct Svg;
#[derive(Debug, Clone)]
pub struct Path;
#[derive(Debug, Clone)]
pub struct Circle;
#[derive(Debug, Clone)]
pub struct Rect;
#[derive(Debug, Clone)]
pub struct Pattern;
#[derive(Debug, Clone)]
pub struct Defs;
#[derive(Debug, Clone)]
pub struct Polygon;

pub fn document() -> Element {
    js::get_document()
}

pub fn query_id(id: &str) -> Option<Element> {
    js::get_element_by_id(id)
}

pub fn wrap<T>(element: Element) -> DomBuilder<T> {
    DomBuilder::new(element)
}

pub fn div() -> DomBuilder<Div> {
    DomBuilder::new(js::create_div())
}

pub fn button() -> DomBuilder<Button> {
    DomBuilder::new(js::create_button())
}

pub fn input() -> DomBuilder<Input> {
    DomBuilder::new(js::create_input())
}

pub fn textarea() -> DomBuilder<TextArea> {
    DomBuilder::new(js::create_textarea())
}

pub fn svg() -> DomBuilder<Svg> {
    DomBuilder::new(js::create_svg())
}

pub fn path() -> DomBuilder<Path> {
    DomBuilder::new(js::create_path())
}

pub fn circle() -> DomBuilder<Circle> {
    DomBuilder::new(js::create_circle())
}

pub fn rect() -> DomBuilder<Rect> {
    DomBuilder::new(js::create_rect())
}

pub fn pattern() -> DomBuilder<Pattern> {
    DomBuilder::new(js::create_pattern())
}

pub fn defs() -> DomBuilder<Defs> {
    DomBuilder::new(js::create_defs())
}

pub fn polygon() -> DomBuilder<Polygon> {
    DomBuilder::new(js::create_polygon())
}
