use euclid::Box2D;
use wasm_bindgen::prelude::*;
use web_sys::{Element, js_sys::Array};

use crate::viewport::ClientSpace;

#[wasm_bindgen(module = "/src/dom/js.js")]
extern "C" {
    #[wasm_bindgen(js_name = getDocument)]
    pub fn get_document() -> Element;

    #[wasm_bindgen(js_name = getElementById)]
    pub fn get_element_by_id(id: &str) -> Option<Element>;

    #[wasm_bindgen(js_name = createDiv)]
    pub fn create_div() -> Element;

    #[wasm_bindgen(js_name = createButton)]
    pub fn create_button() -> Element;

    #[wasm_bindgen(js_name = createInput)]
    pub fn create_input() -> Element;

    #[wasm_bindgen(js_name = createTextarea)]
    pub fn create_textarea() -> Element;

    #[wasm_bindgen(js_name = createSvg)]
    pub fn create_svg() -> Element;

    #[wasm_bindgen(js_name = createPath)]
    pub fn create_path() -> Element;

    #[wasm_bindgen(js_name = createCircle)]
    pub fn create_circle() -> Element;

    #[wasm_bindgen(js_name = createRect)]
    pub fn create_rect() -> Element;

    #[wasm_bindgen(js_name = createPattern)]
    pub fn create_pattern() -> Element;

    #[wasm_bindgen(js_name = createDefs)]
    pub fn create_defs() -> Element;

    #[wasm_bindgen(js_name = createPolygon)]
    pub fn create_polygon() -> Element;

    #[wasm_bindgen(js_name = setAttr)]
    pub fn set_attr(element: &Element, key: &str, value: &str);

    #[wasm_bindgen(js_name = removeAttr)]
    pub fn remove_attr(element: &Element, key: &str);

    #[wasm_bindgen(js_name = setClass)]
    pub fn set_class(element: &Element, name: &str);

    #[wasm_bindgen(js_name = setId)]
    pub fn set_id(element: &Element, id: &str);

    #[wasm_bindgen(js_name = setHtml)]
    pub fn set_html(element: &Element, html: &str);

    #[wasm_bindgen(js_name = setText)]
    pub fn set_text(element: &Element, text: &str);

    #[wasm_bindgen(js_name = show)]
    pub fn show(element: &Element);

    #[wasm_bindgen(js_name = hide)]
    pub fn hide(element: &Element);

    #[wasm_bindgen(js_name = setStyle)]
    pub fn set_style(element: &Element, key: &str, value: &str);

    #[wasm_bindgen(js_name = setLeft)]
    pub fn set_left(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setTop)]
    pub fn set_top(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setRight)]
    pub fn set_right(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setBottom)]
    pub fn set_bottom(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setWidth)]
    pub fn set_width(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setHeight)]
    pub fn set_height(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setSize)]
    pub fn set_size(element: &Element, width: f64, height: f64);

    #[wasm_bindgen(js_name = translate)]
    pub fn translate(element: &Element, x: f64, y: f64);

    #[wasm_bindgen(js_name = translateScale)]
    pub fn translate_scale(element: &Element, x: f64, y: f64, scale: f64);

    #[wasm_bindgen(js_name = setViewBox)]
    pub fn set_viewbox(element: &Element, x: f64, y: f64, width: f64, height: f64);

    #[wasm_bindgen(js_name = setPathD)]
    pub fn set_path_d(element: &Element, d: &str);

    #[wasm_bindgen(js_name = setPathBezier)]
    pub fn set_path_bezier(
        element: &Element,
        x1: f64,
        y1: f64,
        cx1: f64,
        cy1: f64,
        cx2: f64,
        cy2: f64,
        x2: f64,
        y2: f64,
    );

    #[wasm_bindgen(js_name = setStroke)]
    pub fn set_stroke(element: &Element, color: &str);

    #[wasm_bindgen(js_name = setStrokeWidth)]
    pub fn set_stroke_width(element: &Element, width: f64);

    #[wasm_bindgen(js_name = setFill)]
    pub fn set_fill(element: &Element, color: &str);

    #[wasm_bindgen(js_name = setStrokeDasharray)]
    pub fn set_stroke_dasharray(element: &Element, pattern: &str);

    #[wasm_bindgen(js_name = setPoints)]
    pub fn set_points(element: &Element, points: &str);

    #[wasm_bindgen(js_name = setCx)]
    pub fn set_cx(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setCy)]
    pub fn set_cy(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setR)]
    pub fn set_r(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setX)]
    pub fn set_x(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setY)]
    pub fn set_y(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setSvgWidth)]
    pub fn set_svg_width(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setSvgHeight)]
    pub fn set_svg_height(element: &Element, value: f64);

    #[wasm_bindgen(js_name = setPatternUnits)]
    pub fn set_pattern_units(element: &Element, units: &str);

    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(element: &Element) -> String;

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(element: &Element, value: &str);

    #[wasm_bindgen(js_name = getChecked)]
    pub fn get_checked(element: &Element) -> bool;

    #[wasm_bindgen(js_name = setChecked)]
    pub fn set_checked(element: &Element, checked: bool);

    #[wasm_bindgen(js_name = setPlaceholder)]
    pub fn set_placeholder(element: &Element, text: &str);

    #[wasm_bindgen(js_name = setType)]
    pub fn set_type(element: &Element, type_: &str);

    #[wasm_bindgen(js_name = focus)]
    pub fn focus(element: &Element);

    #[wasm_bindgen(js_name = blur)]
    pub fn blur(element: &Element);

    #[wasm_bindgen(js_name = getClientRect)]
    fn get_client_rect_raw(element: &Element) -> JsValue;

    #[wasm_bindgen(js_name = getOffsetWidth)]
    pub fn get_offset_width(element: &Element) -> i32;

    #[wasm_bindgen(js_name = getOffsetHeight)]
    pub fn get_offset_height(element: &Element) -> i32;

    #[wasm_bindgen(js_name = appendChild)]
    pub fn append_child(parent: &Element, child: &Element);

    #[wasm_bindgen(js_name = removeElement)]
    pub fn remove_element(element: &Element);

    #[wasm_bindgen(js_name = setTabIndex)]
    pub fn set_tab_index(element: &Element, index: i32);
}

pub fn get_client_rect(element: &Element) -> Box2D<f64, ClientSpace> {
    use crate::viewport::ClientPoint;

    let array = Array::from(&get_client_rect_raw(element));

    let left = array.get(0).as_f64().unwrap_or(0.0);
    let top = array.get(1).as_f64().unwrap_or(0.0);
    let right = array.get(2).as_f64().unwrap_or(0.0);
    let bottom = array.get(3).as_f64().unwrap_or(0.0);

    Box2D::new(
        ClientPoint::new(left as i32, top as i32),
        ClientPoint::new(right as i32, bottom as i32),
    )
    .to_f64()
}
