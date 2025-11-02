#![allow(non_snake_case)]

use std::collections::HashMap;

use igloo_interface::{
    InputID, PenguinNodeDefnRef, PenguinPinID, PenguinPinType, PenguinType, PenguinValue,
    graph::{
        PenguinGraph, PenguinInputValue, PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID,
    },
};
use log::Level;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::app::{APP, PenguinApp};

mod app;
mod context;
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

fn test_graph_2() -> PenguinGraph {
    let mut g = PenguinGraph::default();

    let mut x = 0.;
    let mut y = 0.;

    for i in 0..1000 {
        if i % 50 == 0 {
            x = 0.;
            y += 500.;
        } else {
            x += 250.;
        }

        g.nodes.insert(
            PenguinNodeID(i),
            PenguinNode::new(PenguinNodeDefnRef::new("std", "int_add_3", 1), x, y),
        );

        if i == 0 {
            continue;
        }

        for w in 0..3 {
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

fn test_graph() -> PenguinGraph {
    let mut g = PenguinGraph::default();

    g.nodes.insert(
        PenguinNodeID(0),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "on_start", 1), 180., 900.),
    );

    g.nodes.insert(
        PenguinNodeID(1),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "const_int", 1), -200., 1200.),
    );
    g.nodes.insert(
        PenguinNodeID(2),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "const_int", 1), -200., 1300.),
    );
    g.nodes.insert(
        PenguinNodeID(3),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "const_int", 1), 100., 1340.),
    );

    g.nodes.insert(
        PenguinNodeID(4),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "int_add_2", 1), 20., 1240.),
    );

    g.nodes.insert(
        PenguinNodeID(5),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "int_mul_2", 1), 300., 1220.),
    );

    g.nodes.insert(
        PenguinNodeID(6),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "const_int", 1), 160., 1020.),
    );
    g.nodes.insert(
        PenguinNodeID(7),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "int_gt", 1), 340., 1020.),
    );

    g.nodes.insert(
        PenguinNodeID(8),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "branch", 1), 500., 900.),
    );

    g.nodes.insert(
        PenguinNodeID(9),
        PenguinNode::new(
            PenguinNodeDefnRef::new("std", "cast_integer_to_text", 1),
            640.,
            1200.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(10),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "const_text", 1), 420., 1380.),
    );
    g.nodes.insert(
        PenguinNodeID(11),
        PenguinNode::new(
            PenguinNodeDefnRef::new("std", "text_to_upper", 1),
            640.,
            1260.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(12),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "print", 1), 1020., 800.),
    );

    g.nodes.insert(
        PenguinNodeID(13),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "const_int", 1), 640., 1260.),
    );
    g.nodes.insert(
        PenguinNodeID(14),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "int_sub", 1), 640., 1260.),
    );
    g.nodes.insert(
        PenguinNodeID(15),
        PenguinNode::new(
            PenguinNodeDefnRef::new("std", "cast_integer_to_text", 1),
            840.,
            1260.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(16),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "print", 1), 1020., 1140.),
    );

    g.nodes.insert(
        PenguinNodeID(17),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "merge_2", 1), 1360., 920.),
    );

    g.nodes.insert(
        PenguinNodeID(18),
        PenguinNode::new(
            PenguinNodeDefnRef::new("std", "const_text", 1),
            1260.,
            1240.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(19),
        PenguinNode::new(PenguinNodeDefnRef::new("std", "print", 1), 1680., 940.),
    );

    g.nodes.insert(
        PenguinNodeID(19),
        PenguinNode {
            defn_ref: PenguinNodeDefnRef::new("std", "comment", 1),
            x: 700.,
            y: 500.,
            input_cfg_values: HashMap::from([(
                InputID("Value".to_string()),
                PenguinInputValue::new(PenguinValue::Text("Example Comment".to_string())),
            )]),
            ..Default::default()
        }, // PenguinNode::new(, 700., 500.),
    );

    g.wires.insert(
        PenguinWireID(1),
        PenguinWire {
            from_node: PenguinNodeID(1),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(4),
            to_pin: PenguinPinID::from_str("Input_0"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(2),
        PenguinWire {
            from_node: PenguinNodeID(2),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(4),
            to_pin: PenguinPinID::from_str("Input_1"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(3),
        PenguinWire {
            from_node: PenguinNodeID(4),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(5),
            to_pin: PenguinPinID::from_str("Input_0"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(4),
        PenguinWire {
            from_node: PenguinNodeID(3),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(5),
            to_pin: PenguinPinID::from_str("Input_1"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(5),
        PenguinWire {
            from_node: PenguinNodeID(5),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(7),
            to_pin: PenguinPinID::from_str("A"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(6),
        PenguinWire {
            from_node: PenguinNodeID(6),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(7),
            to_pin: PenguinPinID::from_str("B"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(7),
        PenguinWire {
            from_node: PenguinNodeID(0),
            from_pin: PenguinPinID::from_str("On Trigger"),
            to_node: PenguinNodeID(8),
            to_pin: PenguinPinID::from_str("Call"),
            r#type: PenguinPinType::Flow,
        },
    );
    g.wires.insert(
        PenguinWireID(8),
        PenguinWire {
            from_node: PenguinNodeID(7),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(8),
            to_pin: PenguinPinID::from_str("Condition"),
            r#type: PenguinPinType::Value(PenguinType::Bool),
        },
    );
    g.wires.insert(
        PenguinWireID(9),
        PenguinWire {
            from_node: PenguinNodeID(5),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(9),
            to_pin: PenguinPinID::from_str("Input"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(10),
        PenguinWire {
            from_node: PenguinNodeID(10),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(11),
            to_pin: PenguinPinID::from_str("Input"),
            r#type: PenguinPinType::Value(PenguinType::Text),
        },
    );
    g.wires.insert(
        PenguinWireID(11),
        PenguinWire {
            from_node: PenguinNodeID(11),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(12),
            to_pin: PenguinPinID::from_str("Message"),
            r#type: PenguinPinType::Value(PenguinType::Text),
        },
    );
    g.wires.insert(
        PenguinWireID(12),
        PenguinWire {
            from_node: PenguinNodeID(8),
            from_pin: PenguinPinID::from_str("True"),
            to_node: PenguinNodeID(12),
            to_pin: PenguinPinID::from_str("Execute"),
            r#type: PenguinPinType::Flow,
        },
    );
    g.wires.insert(
        PenguinWireID(13),
        PenguinWire {
            from_node: PenguinNodeID(5),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(14),
            to_pin: PenguinPinID::from_str("A"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(14),
        PenguinWire {
            from_node: PenguinNodeID(13),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(14),
            to_pin: PenguinPinID::from_str("B"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(15),
        PenguinWire {
            from_node: PenguinNodeID(14),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(15),
            to_pin: PenguinPinID::from_str("Input"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(16),
        PenguinWire {
            from_node: PenguinNodeID(15),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(16),
            to_pin: PenguinPinID::from_str("Message"),
            r#type: PenguinPinType::Value(PenguinType::Text),
        },
    );
    g.wires.insert(
        PenguinWireID(17),
        PenguinWire {
            from_node: PenguinNodeID(8),
            from_pin: PenguinPinID::from_str("False"),
            to_node: PenguinNodeID(16),
            to_pin: PenguinPinID::from_str("Execute"),
            r#type: PenguinPinType::Flow,
        },
    );
    g.wires.insert(
        PenguinWireID(18),
        PenguinWire {
            from_node: PenguinNodeID(12),
            from_pin: PenguinPinID::from_str("Done"),
            to_node: PenguinNodeID(17),
            to_pin: PenguinPinID::from_str("Input_0"),
            r#type: PenguinPinType::Flow,
        },
    );
    g.wires.insert(
        PenguinWireID(19),
        PenguinWire {
            from_node: PenguinNodeID(16),
            from_pin: PenguinPinID::from_str("Done"),
            to_node: PenguinNodeID(17),
            to_pin: PenguinPinID::from_str("Input_1"),
            r#type: PenguinPinType::Flow,
        },
    );

    g
}
