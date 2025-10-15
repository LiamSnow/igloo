use igloo_interface::{
    Component, ComponentType, QueryFilter, QueryTarget, SetQuery, dash::Dashboard,
    ws::ClientMessage,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlInputElement, WebSocket};

use crate::{document, elements::RenderElement};

pub trait RenderDash {
    fn render(&self, dash_id: u16, ws: &WebSocket) -> Result<(), String>;
}

impl RenderDash for Dashboard {
    fn render(&self, dash_id: u16, ws: &WebSocket) -> Result<(), String> {
        let main = document().get_element_by_id("main").expect("missing main");
        let mut elid = (dash_id as u32) << 16;
        let (markup, handlers) = self.child.render(&mut elid, &self.targets)?;

        main.set_inner_html(&markup.into_string());

        for handler in handlers {
            let Some(el) = document().get_element_by_id(&handler.elid.to_string()) else {
                log::error!("Missing element {}", handler.elid);
                continue;
            };

            let Some(input) = el.dyn_ref::<HtmlInputElement>() else {
                log::error!("Element {} is not an input", handler.elid);
                continue;
            };

            let onchange = create_input_handler(
                ws.clone(),
                handler.comp_type,
                handler.filter.clone(),
                handler.target.clone(),
            );

            input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
            onchange.forget();
        }

        Ok(())
    }
}

fn create_input_handler(
    ws: WebSocket,
    comp_type: ComponentType,
    filter: QueryFilter,
    target: QueryTarget,
) -> Closure<dyn FnMut(web_sys::Event)> {
    Closure::new(move |event: web_sys::Event| {
        let input = event
            .target()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        let value = input.value();

        match Component::from_string(comp_type, &value) {
            Ok(component) => {
                let msg: ClientMessage = SetQuery {
                    filter: filter.clone(),
                    target: target.clone(),
                    values: vec![component],
                }
                .into();

                match borsh::to_vec(&msg) {
                    Ok(bytes) => {
                        if let Err(e) = ws.send_with_u8_array(&bytes) {
                            log::error!("Failed to send: {:?}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to serialize: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to parse component: {:?}", e);
            }
        }
    })
}
