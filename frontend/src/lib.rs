mod shared;

use igloo_interface::Component;
use igloo_interface::ComponentType;
use maud::Markup;
use maud::html;
use rustc_hash::FxHashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use web_sys::HtmlInputElement;
use web_sys::console;
use web_sys::js_sys;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::shared::Dashboard;
use crate::shared::Element;
use crate::shared::Imsg;
use crate::shared::QueryFilter;
use crate::shared::QueryTarget;
use crate::shared::SetQuery;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    connect_websocket().unwrap();

    Ok(())
}

pub fn connect_websocket() -> Result<(), JsValue> {
    let ws = WebSocket::new("ws://localhost:3000/ws")?;

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let ws_for_render = ws.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
            let bytes: Vec<u8> = uint8_array.to_vec();

            console::log_1(&format!("Received {} bytes: {:?}", bytes.len(), bytes).into());

            let msg: Imsg = borsh::from_slice(&bytes).unwrap();
            match msg {
                Imsg::Dash(dash) => {
                    console::log_1(&format!("Rendering Dashboard: {}", dash.name).into());
                    if let Err(e) = dash.render(0, &ws_for_render) {
                        console::error_1(&format!("Error rendering dashboard: {e}").into());
                    }
                }
                Imsg::Update(elid, component) => {
                    let Some(el) = document().get_element_by_id(&elid.to_string()) else {
                        console::error_1(&format!("update for missing element: {elid}").into());
                        return;
                    };

                    let Some(inner) = component.inner_string() else {
                        console::error_1(
                            &format!(
                                "update for component which does not have value: {component:?}"
                            )
                            .into(),
                        );
                        return;
                    };

                    if let Some(input) = el.dyn_ref::<HtmlInputElement>() {
                        input.set_value(&inner);
                    } else {
                        el.set_text_content(Some(&inner));
                    }
                }
                _ => {
                    console::error_1(&"unexpected msg".into());
                }
            }
        } else {
            console::log_1(&"recved unknown msg".into());
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        console::log_1(&format!("Error: {:?}", e).into());
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let ws_clone = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        console::log_1(&"Connected!".into());
        let msg = Imsg::GetDash(0);
        let v = borsh::to_vec(&msg).unwrap();
        ws_clone.send_with_u8_array(&v).unwrap();
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}

fn document() -> Document {
    web_sys::window()
        .expect("missing window")
        .document()
        .expect("missing document")
}

impl Dashboard {
    fn render(&self, dash_id: u16, ws: &WebSocket) -> Result<(), String> {
        let main = document().get_element_by_id("main").expect("missing main");
        let mut elid = (dash_id as u32) << 16;
        let (markup, handlers) = self.child.render(&mut elid, &self.targets)?;

        main.set_inner_html(&markup.into_string());

        // Attach handlers after HTML is set
        for (element_id, (filter, target, comp_type)) in handlers {
            let Some(el) = document().get_element_by_id(&element_id.to_string()) else {
                console::error_1(&format!("Missing element {}", element_id).into());
                continue;
            };

            let Some(input) = el.dyn_ref::<HtmlInputElement>() else {
                console::error_1(&format!("Element {} is not an input", element_id).into());
                continue;
            };

            let ws_clone = ws.clone();
            let onchange = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();

                match Component::from_string(comp_type, &value) {
                    Ok(component) => {
                        let msg = Imsg::Set(SetQuery {
                            filter: filter.clone(),
                            target: target.clone(),
                            values: vec![component],
                        });

                        match borsh::to_vec(&msg) {
                            Ok(bytes) => {
                                if let Err(e) = ws_clone.send_with_u8_array(&bytes) {
                                    console::error_1(&format!("Failed to send: {:?}", e).into());
                                }
                            }
                            Err(e) => {
                                console::error_1(&format!("Failed to serialize: {}", e).into());
                            }
                        }
                    }
                    Err(e) => {
                        console::error_1(&format!("Failed to parse component: {:?}", e).into());
                    }
                }
            });

            input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
            onchange.forget();
        }

        Ok(())
    }
}

type HandlerData = Vec<(u32, (QueryFilter, QueryTarget, ComponentType))>;

impl Element {
    fn render(
        &self,
        elid: &mut u32,
        targets: &FxHashMap<String, QueryTarget>,
    ) -> Result<(Markup, HandlerData), String> {
        Ok(match self {
            Element::HStack {
                justify,
                align,
                scroll,
                children,
            } => {
                let overflow = if *scroll { "overflow-x: auto;" } else { "" };
                let css = format!(
                    "display: flex; flex-direction: row; justify-content: {}; align-items: {}; {}",
                    justify, align, overflow
                );

                let mut all_handlers = Vec::new();
                let mut child_markups = Vec::new();

                for child in children {
                    let (markup, handlers) = child.render(elid, targets)?;
                    child_markups.push(markup);
                    all_handlers.extend(handlers);
                }

                let markup = html! {
                    div style=(css) {
                        @for child_markup in child_markups {
                            (child_markup)
                        }
                    }
                };

                (markup, all_handlers)
            }

            Element::VStack {
                justify,
                align,
                scroll,
                children,
            } => {
                let overflow = if *scroll { "overflow-y: auto;" } else { "" };
                let css = format!(
                    "display: flex; flex-direction: column; justify-content: {}; align-items: {}; {}",
                    justify, align, overflow
                );

                let mut all_handlers = Vec::new();
                let mut child_markups = Vec::new();

                for child in children {
                    let (markup, handlers) = child.render(elid, targets)?;
                    child_markups.push(markup);
                    all_handlers.extend(handlers);
                }

                let markup = html! {
                    div style=(css) {
                        @for child_markup in child_markups {
                            (child_markup)
                        }
                    }
                };

                (markup, all_handlers)
            }

            Element::Slider {
                binding,
                disable_validation,
                min,
                max,
                step,
            } => {
                let min_val = min.as_ref().and_then(|c| match c {
                    Component::Int(v) => Some(v.to_string()),
                    Component::Float(v) => Some(v.to_string()),
                    _ => None,
                });

                let max_val = max.as_ref().and_then(|c| match c {
                    Component::Int(v) => Some(v.to_string()),
                    Component::Float(v) => Some(v.to_string()),
                    _ => None,
                });

                let step_val = match step {
                    Some(Component::Int(v)) => v.to_string(),
                    Some(Component::Float(v)) => v.to_string(),
                    _ => "any".to_string(),
                };

                let filter = binding.1.clone();
                let target = targets
                    .get(&binding.0)
                    .ok_or(format!("Missing {}", binding.0))?
                    .clone();
                let comp_type = binding.2;

                let this_elid = *elid;
                *elid += 1;

                let markup = html! {
                    input type="range"
                        min=[min_val]
                        max=[max_val]
                        step=(step_val)
                        id=(this_elid);
                };

                let handlers = vec![(this_elid, (filter, target, comp_type))];

                (markup, handlers)
            }

            _ => todo!(),
        })
    }
}
