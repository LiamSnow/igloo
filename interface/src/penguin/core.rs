use super::*;
use std::collections::HashMap;

pub fn std_library() -> PenguinLibrary {
    let mut nodes = HashMap::new();

    nodes.insert(
        "comment".to_string(),
        NodeDefn {
            title: "Comment".to_string(),
            desc: "Comment".to_string(),
            outputs: vec![],
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: ValueType::Text,
                id: 0,
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "on_start".to_string(),
        NodeDefn {
            title: "On Start".to_string(),
            style: NodeStyle::normal(""),
            desc: "Triggers when program starts".to_string(),
            outputs: vec![PinDefn::new("", PinDefnType::Flow)],
            ..Default::default()
        },
    );

    nodes.insert(
        "print".to_string(),
        NodeDefn {
            title: "Print".to_string(),
            style: NodeStyle::normal(""),
            desc: "Prints text to console".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Flow),
                PinDefn::new("Message", PinDefnType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Flow)],
            ..Default::default()
        },
    );

    nodes.insert(
        "branch".to_string(),
        NodeDefn {
            title: "Branch".to_string(),
            style: NodeStyle::normal(""),
            desc: "Conditionally split flow".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Flow),
                PinDefn::new("Condition", PinDefnType::Value(ValueType::Bool)),
            ],
            outputs: vec![
                PinDefn::new("", PinDefnType::Flow),
                PinDefn::new("", PinDefnType::Flow),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "merge".to_string(),
        NodeDefn {
            title: "Merge".to_string(),
            style: NodeStyle::normal(""),
            desc: "Execute once all branches have completed".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Flow)],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Flow,
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "either".to_string(),
        NodeDefn {
            title: "Either".to_string(),
            style: NodeStyle::normal(""),
            desc: "Execute if either branch triggers".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Flow)],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Flow,
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "const_text".to_string(),
        NodeDefn {
            title: "Text Constant".to_string(),
            desc: "Text constant".to_string(),
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: ValueType::Text,
                id: 0,
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_bool".to_string(),
        NodeDefn {
            title: "Boolean Constant".to_string(),
            desc: "Boolean constant".to_string(),
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: ValueType::Bool,
                id: 0,
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_int".to_string(),
        NodeDefn {
            title: "Integer Constant".to_string(),
            desc: "Integer constant".to_string(),
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: ValueType::Int,
                id: 0,
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_real".to_string(),
        NodeDefn {
            title: "Real Constant".to_string(),
            desc: "Real number constant".to_string(),
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Real))],
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: ValueType::Real,
                id: 0,
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_color".to_string(),
        NodeDefn {
            title: "Color Constant".to_string(),
            desc: "Color constant".to_string(),
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: ValueType::Color,
                id: 0,
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "and".to_string(),
        NodeDefn {
            title: "AND".to_string(),
            style: NodeStyle::background("AND"),
            desc: "Logical AND".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Value(ValueType::Bool),
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "or".to_string(),
        NodeDefn {
            title: "OR".to_string(),
            style: NodeStyle::background("OR"),
            desc: "Logical OR".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Value(ValueType::Bool),
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "not".to_string(),
        NodeDefn {
            title: "NOT".to_string(),
            style: NodeStyle::background("NOT"),
            desc: "Logical NOT".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "xor".to_string(),
        NodeDefn {
            title: "XOR".to_string(),
            style: NodeStyle::background("XOR"),
            desc: "Logical XOR".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Bool)),
                PinDefn::new("", PinDefnType::Value(ValueType::Bool)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "bool_equal".to_string(),
        NodeDefn {
            title: "Boolean Equal".to_string(),
            style: NodeStyle::background("=="),
            desc: "Boolean equality".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Bool)),
                PinDefn::new("", PinDefnType::Value(ValueType::Bool)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_add".to_string(),
        NodeDefn {
            title: "Add Integers".to_string(),
            style: NodeStyle::background("+"),
            desc: "Add integers".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Value(ValueType::Int),
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "int_sub".to_string(),
        NodeDefn {
            title: "Subtract Integers".to_string(),
            style: NodeStyle::background("-"),
            desc: "Subtract integers".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_mul".to_string(),
        NodeDefn {
            title: "Multiply Integers".to_string(),
            style: NodeStyle::background("*"),
            desc: "Multiply integers".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Value(ValueType::Int),
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "int_div".to_string(),
        NodeDefn {
            title: "Divide Integers".to_string(),
            style: NodeStyle::background("/"),
            desc: "Divide integers".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_mod".to_string(),
        NodeDefn {
            title: "Integer Modulo".to_string(),
            style: NodeStyle::background("MOD"),
            desc: "Integer remainder".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_lt".to_string(),
        NodeDefn {
            title: "Integer Less Than".to_string(),
            style: NodeStyle::background("<"),
            desc: "A < B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_gt".to_string(),
        NodeDefn {
            title: "Integer Greater Than".to_string(),
            style: NodeStyle::background(">"),
            desc: "A > B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_lte".to_string(),
        NodeDefn {
            title: "Integer Less Than or Equal".to_string(),
            style: NodeStyle::background("<="),
            desc: "A <= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_gte".to_string(),
        NodeDefn {
            title: "Integer Greater Than or Equal".to_string(),
            style: NodeStyle::background(">="),
            desc: "A >= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_eq".to_string(),
        NodeDefn {
            title: "Integer Equal".to_string(),
            style: NodeStyle::background("=="),
            desc: "A == B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_neq".to_string(),
        NodeDefn {
            title: "Integer Not Equal".to_string(),
            style: NodeStyle::background("!="),
            desc: "A != B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_add".to_string(),
        NodeDefn {
            title: "Add Reals".to_string(),
            style: NodeStyle::background("+"),
            desc: "Add reals".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Real))],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Value(ValueType::Real),
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "real_sub".to_string(),
        NodeDefn {
            title: "Subtract Reals".to_string(),
            style: NodeStyle::background("-"),
            desc: "Subtract reals".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_mul".to_string(),
        NodeDefn {
            title: "Multiply Reals".to_string(),
            style: NodeStyle::background("*"),
            desc: "Multiply reals".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Real))],
            cfg: vec![NodeConfig::AddRemovePin(AddRemovePinConfig {
                r#type: PinType::Value(ValueType::Real),
                phantom_id: 0,
                min: 2,
                max: 10,
            })],
        },
    );

    nodes.insert(
        "real_div".to_string(),
        NodeDefn {
            title: "Divide Reals".to_string(),
            style: NodeStyle::background("/"),
            desc: "Divide reals".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_lt".to_string(),
        NodeDefn {
            title: "Real Less Than".to_string(),
            style: NodeStyle::background("<"),
            desc: "A < B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_gt".to_string(),
        NodeDefn {
            title: "Real Greater Than".to_string(),
            style: NodeStyle::background(">"),
            desc: "A > B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_lte".to_string(),
        NodeDefn {
            title: "Real Less Than or Equal".to_string(),
            style: NodeStyle::background("<="),
            desc: "A <= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_gte".to_string(),
        NodeDefn {
            title: "Real Greater Than or Equal".to_string(),
            style: NodeStyle::background(">="),
            desc: "A >= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_eq".to_string(),
        NodeDefn {
            title: "Real Equal".to_string(),
            style: NodeStyle::background("=="),
            desc: "A == B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_neq".to_string(),
        NodeDefn {
            title: "Real Not Equal".to_string(),
            style: NodeStyle::background("!="),
            desc: "A != B".to_string(),
            inputs: vec![
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_mix".to_string(),
        NodeDefn {
            title: "Mix".to_string(),
            style: NodeStyle::normal(""),
            desc: "Blend two colors".to_string(),
            inputs: vec![
                PinDefn::new("A", PinDefnType::Value(ValueType::Color)),
                PinDefn::new("B", PinDefnType::Value(ValueType::Color)),
                PinDefn::new("Ratio", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_rgb".to_string(),
        NodeDefn {
            title: "From RGB".to_string(),
            style: NodeStyle::normal(""),
            desc: "Create from RGB".to_string(),
            inputs: vec![
                PinDefn::new("R", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("G", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("B", PinDefnType::Value(ValueType::Int)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_rgb".to_string(),
        NodeDefn {
            title: "To RGB".to_string(),
            style: NodeStyle::normal(""),
            desc: "Extract RGB".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            outputs: vec![
                PinDefn::new("R", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("G", PinDefnType::Value(ValueType::Int)),
                PinDefn::new("B", PinDefnType::Value(ValueType::Int)),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_hsl".to_string(),
        NodeDefn {
            title: "From HSL".to_string(),
            style: NodeStyle::normal(""),
            desc: "Create from HSL".to_string(),
            inputs: vec![
                PinDefn::new("H", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("S", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("L", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_hsl".to_string(),
        NodeDefn {
            title: "To HSL".to_string(),
            style: NodeStyle::normal(""),
            desc: "Extract HSL".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            outputs: vec![
                PinDefn::new("H", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("S", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("L", PinDefnType::Value(ValueType::Real)),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_hsv".to_string(),
        NodeDefn {
            title: "From HSV".to_string(),
            style: NodeStyle::normal(""),
            desc: "Create from HSV".to_string(),
            inputs: vec![
                PinDefn::new("H", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("S", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("V", PinDefnType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_hsv".to_string(),
        NodeDefn {
            title: "To HSV".to_string(),
            style: NodeStyle::normal(""),
            desc: "Extract HSV".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Color))],
            outputs: vec![
                PinDefn::new("H", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("S", PinDefnType::Value(ValueType::Real)),
                PinDefn::new("V", PinDefnType::Value(ValueType::Real)),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_length".to_string(),
        NodeDefn {
            title: "Length".to_string(),
            style: NodeStyle::normal(""),
            desc: "Text length".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Int))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_to_upper".to_string(),
        NodeDefn {
            title: "To Uppercase".to_string(),
            style: NodeStyle::background("→"),
            desc: "Convert to uppercase".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_to_lower".to_string(),
        NodeDefn {
            title: "To Lowercase".to_string(),
            style: NodeStyle::background("→"),
            desc: "Convert to lowercase".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_replace".to_string(),
        NodeDefn {
            title: "Replace".to_string(),
            style: NodeStyle::normal(""),
            desc: "Replace text".to_string(),
            inputs: vec![
                PinDefn::new("Text", PinDefnType::Value(ValueType::Text)),
                PinDefn::new("Find", PinDefnType::Value(ValueType::Text)),
                PinDefn::new("Replace", PinDefnType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_regex_match".to_string(),
        NodeDefn {
            title: "Regex Match".to_string(),
            style: NodeStyle::normal(""),
            desc: "Match regex pattern".to_string(),
            inputs: vec![
                PinDefn::new("Text", PinDefnType::Value(ValueType::Text)),
                PinDefn::new("Pattern", PinDefnType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinDefnType::Value(ValueType::Bool))],
            ..Default::default()
        },
    );

    add_cast(&mut nodes, ValueType::Text, ValueType::Int);
    add_cast(&mut nodes, ValueType::Real, ValueType::Int);
    add_cast(&mut nodes, ValueType::Bool, ValueType::Int);

    add_cast(&mut nodes, ValueType::Text, ValueType::Real);
    add_cast(&mut nodes, ValueType::Int, ValueType::Real);
    add_cast(&mut nodes, ValueType::Bool, ValueType::Real);

    add_cast(&mut nodes, ValueType::Text, ValueType::Bool);
    add_cast(&mut nodes, ValueType::Int, ValueType::Bool);
    add_cast(&mut nodes, ValueType::Real, ValueType::Bool);

    add_cast(&mut nodes, ValueType::Int, ValueType::Text);
    add_cast(&mut nodes, ValueType::Real, ValueType::Text);
    add_cast(&mut nodes, ValueType::Bool, ValueType::Text);
    add_cast(&mut nodes, ValueType::Color, ValueType::Text);

    PenguinLibrary {
        name: "std".to_string(),
        nodes,
    }
}

fn add_cast(nodes: &mut HashMap<String, NodeDefn>, from: ValueType, to: ValueType) {
    nodes.insert(
        from.cast_name(to).unwrap(),
        NodeDefn {
            title: format!("Cast {from} to {to}"),
            style: NodeStyle::background("→"),
            desc: "Casting will TODO".to_string(),
            inputs: vec![PinDefn::new("", PinDefnType::Value(from))],
            outputs: vec![PinDefn::new("", PinDefnType::Value(to))],
            ..Default::default()
        },
    );
}
