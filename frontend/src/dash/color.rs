use crate::ws::{send_msg, CURRENT_DASHBOARD, ELEMENT_VALUES};
use dioxus::prelude::*;
use igloo_interface::{
    dash::{ColorPickerElement, ColorPickerVariant},
    Color, Component, QueryFilter, QueryTarget, SetQuery,
};

#[component]
pub(crate) fn ColorPicker(el: ColorPickerElement) -> Element {
    let watch_id = el.watch_id.unwrap();
    let filter = el.binding.filter.clone();
    let dash = CURRENT_DASHBOARD.read();
    let targets = &dash.as_ref().unwrap().targets;
    let target = targets.get(&el.binding.target).unwrap().clone();

    let color = use_memo(move || {
        let vals = ELEMENT_VALUES.read();
        vals.get(&watch_id).and_then(|c| match c {
            Component::Color(c) => Some(c.clone()),
            _ => None,
        })
    });

    match el.variant {
        ColorPickerVariant::RedSlider => {
            rsx! {
                RedSlider { color, filter, target }
            }
        }
        ColorPickerVariant::GreenSlider => {
            rsx! {
                GreenSlider { color, filter, target }
            }
        }
        ColorPickerVariant::BlueSlider => {
            rsx! {
                BlueSlider { color, filter, target }
            }
        }
        ColorPickerVariant::HueSlider => {
            rsx! {
                HueSlider { color, filter, target }
            }
        }
        ColorPickerVariant::SaturationSlider => {
            rsx! {
                SaturationSlider { color, filter, target }
            }
        }
        ColorPickerVariant::ValueSlider => {
            rsx! {
                ValueSlider { color, filter, target }
            }
        }
        ColorPickerVariant::ColorWheel => {
            rsx! {
                ColorWheel { color, filter, target }
            }
        }
        ColorPickerVariant::Square => {
            rsx! {
                ColorSquare { color, filter, target }
            }
        }
    }
}

#[component]
fn RedSlider(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let red = use_memo(move || color().map(|c| c.r));
    rsx! {
        div {
            class: "color-picker-box slider-box",
            input {
                class: "color-picker red-slider",
                r#type: "range",
                min: 0,
                max: 255,
                step: 1,
                value: red,
                onchange: move |event| {
                    let red = event.value().parse().unwrap_or(0.0);
                    let mut color = color().unwrap_or(Color { r: 0, g: 0, b: 0 });
                    color.r = red as u8;
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(color)]
                    }.into());
                }
            }
        }
    }
}

#[component]
fn GreenSlider(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let green = use_memo(move || color().map(|c| c.g));
    rsx! {
        div {
            class: "color-picker-box slider-box",
            input {
                class: "color-picker green-slider",
                r#type: "range",
                min: 0,
                max: 255,
                step: 1,
                value: green,
                onchange: move |event| {
                    let green = event.value().parse().unwrap_or(0.0);
                    let mut color = color().unwrap_or(Color { r: 0, g: 0, b: 0 });
                    color.g = green as u8;
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(color)]
                    }.into());
                }
            }
        }
    }
}

#[component]
fn BlueSlider(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let blue = use_memo(move || color().map(|c| c.b));
    rsx! {
        div {
            class: "color-picker-box slider-box",
            input {
                class: "color-picker blue-slider",
                r#type: "range",
                min: 0,
                max: 255,
                step: 1,
                value: blue,
                onchange: move |event| {
                    let blue = event.value().parse().unwrap_or(0.0);
                    let mut color = color().unwrap_or(Color { r: 0, g: 0, b: 0 });
                    color.b = blue as u8;
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(color)]
                    }.into());
                }
            }
        }
    }
}

#[component]
fn HueSlider(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let hsv = use_memo(move || color().map(|c| rgb_to_hsv(&c)));
    let hue = use_memo(move || hsv().map(|hsv| hsv.0));
    rsx! {
        div {
            class: "color-picker-box slider-box",
            input {
                class: "color-picker hue-slider",
                r#type: "range",
                min: 0,
                max: 360,
                step: "any",
                value: hue,
                onchange: move |event| {
                    let hue = event.value().parse().unwrap_or(0.0);
                    let hsv = hsv().unwrap_or((0., 1.0, 1.0));
                    let color = hsv_to_rgb(hue, hsv.1, hsv.2);
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(color)]
                    }.into());
                }
            }
        }
    }
}

#[component]
fn SaturationSlider(
    color: Memo<Option<Color>>,
    filter: QueryFilter,
    target: QueryTarget,
) -> Element {
    let hsv = use_memo(move || color().map(|c| rgb_to_hsv(&c)));
    let saturation = use_memo(move || hsv().map(|hsv| (hsv.1 * 100.0) as i32));

    let style = use_memo(move || {
        hsv()
            .map(|hsv| {
                let color_min = hsv_to_rgb(hsv.0, 0.0, hsv.2);
                let color_max = hsv_to_rgb(hsv.0, 1.0, hsv.2);
                format!(
                    "background: linear-gradient(to right, rgb({},{},{}), rgb({},{},{}));",
                    color_min.r, color_min.g, color_min.b, color_max.r, color_max.g, color_max.b
                )
            })
            .unwrap_or_default()
    });

    rsx! {
        div {
            class: "color-picker-box slider-box",
            input {
                class: "color-picker saturation-slider",
                r#type: "range",
                min: 0,
                max: 100,
                step: "any",
                value: saturation,
                style: style,
                onchange: move |event| {
                    let sat = event.value().parse().unwrap_or(0.0) / 100.0;
                    let hsv = hsv().unwrap_or((0., 1.0, 1.0));
                    let color = hsv_to_rgb(hsv.0, sat, hsv.2);
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(color)]
                    }.into());
                }
            }
        }
    }
}

#[component]
fn ValueSlider(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let hsv = use_memo(move || color().map(|c| rgb_to_hsv(&c)));
    let value = use_memo(move || hsv().map(|hsv| (hsv.2 * 100.0) as i32));

    let style = use_memo(move || {
        hsv()
            .map(|hsv| {
                let color_min = hsv_to_rgb(hsv.0, hsv.1, 0.0);
                let color_max = hsv_to_rgb(hsv.0, hsv.1, 1.0);
                format!(
                    "background: linear-gradient(to right, rgb({},{},{}), rgb({},{},{}));",
                    color_min.r, color_min.g, color_min.b, color_max.r, color_max.g, color_max.b
                )
            })
            .unwrap_or_default()
    });

    rsx! {
        div {
            class: "color-picker-box slider-box",
            input {
                class: "color-picker value-slider",
                r#type: "range",
                min: 0,
                max: 100,
                value: value,
                style: style,
                onchange: move |event| {
                    let val = event.value().parse().unwrap_or(0.0) / 100.0;
                    let hsv = hsv().unwrap_or((0., 1.0, 1.0));
                    let color = hsv_to_rgb(hsv.0, hsv.1, val);
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(color)]
                    }.into());
                }
            }
        }
    }
}

#[component]
fn ColorWheel(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let hsv = use_memo(move || color().map(|c| rgb_to_hsv(&c)));
    const SIZE: f64 = 200.0;
    const RADIUS: f64 = SIZE / 2.0;

    let style = use_memo(move || {
        let (h, s, _) = hsv().unwrap_or((0.0, 0.0, 1.0));
        let angle = h * std::f64::consts::PI / 180.0;
        let r = s * 50.0;
        format!(
            "left: {}%; top: {}%; transform: translate(-50%, -50%);",
            50.0 + r * angle.cos(),
            50.0 + r * angle.sin()
        )
    });

    rsx! {
        div {
            class: "color-picker-box color-wheel-box",
            div {
                class: "color-picker color-wheel",
                onclick: move |e| {
                    let pos = e.element_coordinates();
                    let dx = pos.x - RADIUS;
                    let dy = pos.y - RADIUS;
                    let h = ((dy.atan2(dx) * 180.0 / std::f64::consts::PI) + 360.0) % 360.0;
                    let s = (dx.hypot(dy) / RADIUS).min(1.0);
                    let v = hsv().map(|(_, _, v)| v).unwrap_or(1.0);
                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(hsv_to_rgb(h, s, v))]
                    }.into());
                },
                div {
                    class: "color-wheel-indicator",
                    style: style,
                }
            }
        }
    }
}

#[component]
fn ColorSquare(color: Memo<Option<Color>>, filter: QueryFilter, target: QueryTarget) -> Element {
    let hsv = use_memo(move || color().map(|c| rgb_to_hsv(&c)));
    const SIZE: f64 = 200.0;

    let (h, _, _) = hsv().unwrap_or((0.0, 1.0, 1.0));

    let style = use_memo(move || {
        let (_, s, v) = hsv().unwrap_or((0.0, 1.0, 1.0));
        format!("left: {}%; top: {}%;", s * 100.0, (1.0 - v) * 100.0)
    });

    let bg_style = use_memo(move || {
        let (h, _, _) = hsv().unwrap_or((0.0, 1.0, 1.0));
        format!("background-color: hsl({}, 100%, 50%);", h)
    });

    rsx! {
        div {
            class: "color-picker-box color-square-box",
            div {
                class: "color-picker color-square",
                style: bg_style,
                onclick: move |e| {
                    let pos = e.element_coordinates();

                    let s = (pos.x / SIZE).clamp(0.0, 1.0);
                    let v = 1.0 - (pos.y / SIZE).clamp(0.0, 1.0);

                    send_msg(SetQuery {
                        filter: filter.clone(),
                        target: target.clone(),
                        values: vec![Component::Color(hsv_to_rgb(h, s, v))]
                    }.into());
                },
                div {
                    class: "color-square-indicator",
                    style: style,
                }
            }
        }
    }
}

fn rgb_to_hsv(color: &Color) -> (f64, f64, f64) {
    let r = color.r as f64 / 255.0;
    let g = color.g as f64 / 255.0;
    let b = color.b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;

    if delta == 0.0 {
        return (0.0, 0.0, v);
    }

    let s = delta / max;

    let h = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Color {
    if s == 0.0 {
        let gray = (v * 255.0).round() as u8;
        return Color {
            r: gray,
            g: gray,
            b: gray,
        };
    }

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color {
        r: ((r + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        g: ((g + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        b: ((b + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    }
}
