extern crate console_error_panic_hook;
use crate::app::APP;
use igloo_interface::{
    NodeInputFeatureID, PenguinNodeDefnRef, PenguinPinID, PenguinPinType, PenguinType,
    PenguinValue,
    graph::{
        PenguinGraph, PenguinInputValue, PenguinNode, PenguinNodeID, PenguinWire, PenguinWireID,
    },
};
use log::Level;
use std::{collections::HashMap, panic};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

mod app;
mod graph;
mod menu;
mod viewport;

#[wasm_bindgen(start)]
fn init() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).unwrap();
    log::info!("Penguin Initialized");
}

#[wasm_bindgen]
pub fn penguin_start() -> Result<(), JsValue> {
    log::info!("Starting Penguin");

    app::App::init()?;

    APP.with(|a| {
        let mut b = a.borrow_mut();
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

    g.nodes.insert(
        PenguinNodeID(0),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "On Start", 1),
            180.,
            900.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(1),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Integer Constant", 1),
            -200.,
            1200.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(2),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Integer Constant", 1),
            -200.,
            1300.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(3),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Integer Constant", 1),
            100.,
            1340.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(4),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Add Integers 2", 1),
            20.,
            1240.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(5),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Multiply Integers 2", 1),
            300.,
            1220.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(6),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Integer Constant", 1),
            160.,
            1020.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(7),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Integer Greater Than", 1),
            340.,
            1020.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(8),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Branch", 1),
            500.,
            900.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(9),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Cast Integer to Text", 1),
            640.,
            1200.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(10),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Text Constant", 1),
            420.,
            1380.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(11),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Text to Uppercase", 1),
            640.,
            1260.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(12),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Print", 1),
            1020.,
            800.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(13),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Integer Constant", 1),
            640.,
            1260.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(14),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Subtract Integers", 1),
            640.,
            1260.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(15),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Cast Integer to Text", 1),
            840.,
            1260.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(16),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Print", 1),
            1020.,
            1140.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(17),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Merge 2", 1),
            1360.,
            920.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(18),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Text Constant", 1),
            1260.,
            1240.,
        ),
    );
    g.nodes.insert(
        PenguinNodeID(19),
        PenguinNode::new(
            PenguinNodeDefnRef::new("Standard Library", "Print", 1),
            1680.,
            940.,
        ),
    );

    g.nodes.insert(
        PenguinNodeID(19),
        PenguinNode {
            defn_ref: PenguinNodeDefnRef::new("Standard Library", "Comment", 1),
            x: 700.,
            y: 500.,
            input_feature_values: HashMap::from([(
                NodeInputFeatureID("Value".to_string()),
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
            to_pin: PenguinPinID::from_str("Input 0"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(2),
        PenguinWire {
            from_node: PenguinNodeID(2),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(4),
            to_pin: PenguinPinID::from_str("Input 1"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(3),
        PenguinWire {
            from_node: PenguinNodeID(4),
            from_pin: PenguinPinID::from_str("Output"),
            to_node: PenguinNodeID(5),
            to_pin: PenguinPinID::from_str("Input 0"),
            r#type: PenguinPinType::Value(PenguinType::Int),
        },
    );
    g.wires.insert(
        PenguinWireID(4),
        PenguinWire {
            from_node: PenguinNodeID(3),
            from_pin: PenguinPinID::from_str("Value"),
            to_node: PenguinNodeID(5),
            to_pin: PenguinPinID::from_str("Input 1"),
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
            to_pin: PenguinPinID::from_str("Input 0"),
            r#type: PenguinPinType::Flow,
        },
    );
    g.wires.insert(
        PenguinWireID(19),
        PenguinWire {
            from_node: PenguinNodeID(16),
            from_pin: PenguinPinID::from_str("Done"),
            to_node: PenguinNodeID(17),
            to_pin: PenguinPinID::from_str("Input 1"),
            r#type: PenguinPinType::Flow,
        },
    );

    g
}
