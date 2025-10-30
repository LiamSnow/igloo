#![allow(non_snake_case)]

use igloo_interface::{
    PenguinNodeDefnRef, PenguinPinID, PenguinPinType, PenguinType,
    graph::{PenguinGraph, PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID},
};
use log::Level;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::app::{APP, PenguinApp};

mod app;
mod ffi;
mod graph;
mod grid;
mod interaction;
mod viewport;

#[wasm_bindgen(start)]
fn init() {
    console_log::init_with_level(Level::Debug).unwrap();
    log::info!("Penguin Initialized");
}

#[wasm_bindgen]
pub fn penguin_start() -> Result<(), JsValue> {
    log::info!("Starting Penguin");

    let app = PenguinApp::new()?;

    APP.with(|a| {
        let mut b = a.borrow_mut();
        *b = Some(app);
        if let Err(e) = b.as_mut().unwrap().load(test_graph()) {
            log::error!("Error loading graph: {e:?}");
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub fn penguin_stop() -> Result<(), JsValue> {
    APP.with(|app| {
        app.borrow_mut().take();
    });

    Ok(())
}

fn test_graph() -> PenguinGraph {
    let mut g = PenguinGraph::default();

    let mut x = 0.;
    let mut y = 0.;

    for i in 0..10 {
        if i % 50 == 0 {
            x = 0.;
            y += 500.;
        } else {
            x += 250.;
            // x += 450.;
        }

        g.nodes.insert(
            PenguinNodeID(i),
            // if i % 2 == 0 {
            //     PenguinNode::new(PenguinNodeDefnRef::new("std", "print", 1), x, y)
            // } else {
            //     PenguinNode::new(PenguinNodeDefnRef::new("std", "const_text", 1), x, y)
            // },
            PenguinNode::new(PenguinNodeDefnRef::new("std", "int_add_5", 1), x, y),
        );

        if i == 0 {
            continue;
        }

        for w in 0..5 {
            g.wires.insert(
                PenguinWireID((w << 8) + i),
                PenguinWire {
                    from_node: PenguinNodeID(i - 1),
                    from_pin: PenguinPinID::from_str("Output"),
                    to_node: PenguinNodeID(i),
                    to_pin: PenguinPinID::from_str(&format!("Input_{w}")),
                    r#type: PenguinPinType::Value(PenguinType::Int),
                },
            );
        }
    }

    g
}
