use super::*;
use std::collections::HashMap;

pub fn std_library() -> PenguinLibrary {
    let mut nodes = HashMap::new();

    nodes.insert(
        "on_start".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "On Start"),
            desc: "Triggers when program starts".to_string(),
            outputs: vec![PinDefn::new("", PinType::Flow)],
            ..Default::default()
        },
    );

    // TODO add query

    nodes.insert(
        "print".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Print"),
            desc: "Prints text to console".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Flow),
                PinDefn::new("Message", PinType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinType::Flow)],
            ..Default::default()
        },
    );

    nodes.insert(
        "branch".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Branch"),
            desc: "Conditionally split flow".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Flow),
                PinDefn::new("Condition", PinType::Value(ValueType::Boolean)),
            ],
            outputs: vec![
                PinDefn::new("", PinType::Flow),
                PinDefn::new("", PinType::Flow),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "merge".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Merge"),
            desc: "Execute once all branches have completed".to_string(),
            inputs: vec![PinDefn::new("", PinType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinType::Flow)],
            cfg: vec![
                NodeConfig::AddPin(AddPinConfig {
                    r#type: PinType::Flow,
                    phantom_id: 0,
                    max: 10,
                }),
                NodeConfig::RemovePin(RemovePinConfig {
                    r#type: PinType::Flow,
                    phantom_id: 0,
                    min: 2,
                }),
            ],
        },
    );

    nodes.insert(
        "either".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Either"),
            desc: "Execute if either branch triggers".to_string(),
            inputs: vec![PinDefn::new("", PinType::Phantom(0))],
            outputs: vec![PinDefn::new("", PinType::Flow)],
            cfg: vec![
                NodeConfig::AddPin(AddPinConfig {
                    r#type: PinType::Flow,
                    phantom_id: 0,
                    max: 10,
                }),
                NodeConfig::RemovePin(RemovePinConfig {
                    r#type: PinType::Flow,
                    phantom_id: 0,
                    min: 2,
                }),
            ],
        },
    );

    nodes.insert(
        "const_text".to_string(),
        NodeDefn {
            desc: "Text constant".to_string(),
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_bool".to_string(),
        NodeDefn {
            desc: "Boolean constant".to_string(),
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_int".to_string(),
        NodeDefn {
            desc: "Integer constant".to_string(),
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_real".to_string(),
        NodeDefn {
            desc: "Real number constant".to_string(),
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_color".to_string(),
        NodeDefn {
            desc: "Color constant".to_string(),
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "and".to_string(),
        NodeDefn {
            style: NodeStyle::compact("AND"),
            desc: "Logical AND".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "or".to_string(),
        NodeDefn {
            style: NodeStyle::compact("OR"),
            desc: "Logical OR".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "not".to_string(),
        NodeDefn {
            style: NodeStyle::compact("NOT"),
            desc: "Logical NOT".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "xor".to_string(),
        NodeDefn {
            style: NodeStyle::compact("XOR"),
            desc: "Logical XOR".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "bool_equal".to_string(),
        NodeDefn {
            style: NodeStyle::compact("=="),
            desc: "Boolean equality".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
                PinDefn::new("", PinType::Value(ValueType::Boolean)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_add".to_string(),
        NodeDefn {
            style: NodeStyle::compact("+"),
            desc: "Add integers".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_sub".to_string(),
        NodeDefn {
            style: NodeStyle::compact("-"),
            desc: "Subtract integers".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_mul".to_string(),
        NodeDefn {
            style: NodeStyle::compact("*"),
            desc: "Multiply integers".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_div".to_string(),
        NodeDefn {
            style: NodeStyle::compact("/"),
            desc: "Divide integers".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_mod".to_string(),
        NodeDefn {
            style: NodeStyle::compact("MOD"),
            desc: "Integer remainder".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_lt".to_string(),
        NodeDefn {
            style: NodeStyle::compact("<"),
            desc: "A < B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_gt".to_string(),
        NodeDefn {
            style: NodeStyle::compact(">"),
            desc: "A > B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_lte".to_string(),
        NodeDefn {
            style: NodeStyle::compact("<="),
            desc: "A <= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_gte".to_string(),
        NodeDefn {
            style: NodeStyle::compact(">="),
            desc: "A >= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_eq".to_string(),
        NodeDefn {
            style: NodeStyle::compact("=="),
            desc: "A == B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_neq".to_string(),
        NodeDefn {
            style: NodeStyle::compact("!="),
            desc: "A != B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Integer)),
                PinDefn::new("", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_add".to_string(),
        NodeDefn {
            style: NodeStyle::compact("+"),
            desc: "Add reals".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_sub".to_string(),
        NodeDefn {
            style: NodeStyle::compact("-"),
            desc: "Subtract reals".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_mul".to_string(),
        NodeDefn {
            style: NodeStyle::compact("*"),
            desc: "Multiply reals".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_div".to_string(),
        NodeDefn {
            style: NodeStyle::compact("/"),
            desc: "Divide reals".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_lt".to_string(),
        NodeDefn {
            style: NodeStyle::compact("<"),
            desc: "A < B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_gt".to_string(),
        NodeDefn {
            style: NodeStyle::compact(">"),
            desc: "A > B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_lte".to_string(),
        NodeDefn {
            style: NodeStyle::compact("<="),
            desc: "A <= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_gte".to_string(),
        NodeDefn {
            style: NodeStyle::compact(">="),
            desc: "A >= B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_eq".to_string(),
        NodeDefn {
            style: NodeStyle::compact("=="),
            desc: "A == B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_neq".to_string(),
        NodeDefn {
            style: NodeStyle::compact("!="),
            desc: "A != B".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Real)),
                PinDefn::new("", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_mix".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Mix"),
            desc: "Blend two colors".to_string(),
            inputs: vec![
                PinDefn::new("A", PinType::Value(ValueType::Color)),
                PinDefn::new("B", PinType::Value(ValueType::Color)),
                PinDefn::new("Ratio", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_rgb".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "From RGB"),
            desc: "Create from RGB".to_string(),
            inputs: vec![
                PinDefn::new("R", PinType::Value(ValueType::Integer)),
                PinDefn::new("G", PinType::Value(ValueType::Integer)),
                PinDefn::new("B", PinType::Value(ValueType::Integer)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_rgb".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "To RGB"),
            desc: "Extract RGB".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            outputs: vec![
                PinDefn::new("R", PinType::Value(ValueType::Integer)),
                PinDefn::new("G", PinType::Value(ValueType::Integer)),
                PinDefn::new("B", PinType::Value(ValueType::Integer)),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_hsl".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "From HSL"),
            desc: "Create from HSL".to_string(),
            inputs: vec![
                PinDefn::new("H", PinType::Value(ValueType::Real)),
                PinDefn::new("S", PinType::Value(ValueType::Real)),
                PinDefn::new("L", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_hsl".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "To HSL"),
            desc: "Extract HSL".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            outputs: vec![
                PinDefn::new("H", PinType::Value(ValueType::Real)),
                PinDefn::new("S", PinType::Value(ValueType::Real)),
                PinDefn::new("L", PinType::Value(ValueType::Real)),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_hsv".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "From HSV"),
            desc: "Create from HSV".to_string(),
            inputs: vec![
                PinDefn::new("H", PinType::Value(ValueType::Real)),
                PinDefn::new("S", PinType::Value(ValueType::Real)),
                PinDefn::new("V", PinType::Value(ValueType::Real)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_hsv".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "To HSV"),
            desc: "Extract HSV".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            outputs: vec![
                PinDefn::new("H", PinType::Value(ValueType::Real)),
                PinDefn::new("S", PinType::Value(ValueType::Real)),
                PinDefn::new("V", PinType::Value(ValueType::Real)),
            ],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_concat".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Concatenate"),
            desc: "Join text".to_string(),
            inputs: vec![
                PinDefn::new("", PinType::Value(ValueType::Text)),
                PinDefn::new("", PinType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_length".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Length"),
            desc: "Text length".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_to_upper".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "To Uppercase"),
            desc: "Convert to uppercase".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_to_lower".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "To Lowercase"),
            desc: "Convert to lowercase".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_replace".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Replace"),
            desc: "Replace text".to_string(),
            inputs: vec![
                PinDefn::new("Text", PinType::Value(ValueType::Text)),
                PinDefn::new("Find", PinType::Value(ValueType::Text)),
                PinDefn::new("Replace", PinType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "text_regex_match".to_string(),
        NodeDefn {
            style: NodeStyle::normal("", "Regex Match"),
            desc: "Match regex pattern".to_string(),
            inputs: vec![
                PinDefn::new("Text", PinType::Value(ValueType::Text)),
                PinDefn::new("Pattern", PinType::Value(ValueType::Text)),
            ],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_to_text".to_string(),
        NodeDefn {
            desc: "Convert integer to text".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_to_text".to_string(),
        NodeDefn {
            desc: "Convert real to text".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "bool_to_text".to_string(),
        NodeDefn {
            desc: "Convert boolean to text".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_text".to_string(),
        NodeDefn {
            desc: "Convert color to text".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Color))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Text))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_to_real".to_string(),
        NodeDefn {
            desc: "Convert integer to real".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            ..Default::default()
        },
    );

    nodes.insert(
        "real_to_int".to_string(),
        NodeDefn {
            desc: "Convert real to integer".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Real))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "bool_to_int".to_string(),
        NodeDefn {
            desc: "Convert boolean to integer".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            ..Default::default()
        },
    );

    nodes.insert(
        "int_to_bool".to_string(),
        NodeDefn {
            desc: "Convert integer to boolean".to_string(),
            inputs: vec![PinDefn::new("", PinType::Value(ValueType::Integer))],
            outputs: vec![PinDefn::new("", PinType::Value(ValueType::Boolean))],
            ..Default::default()
        },
    );

    PenguinLibrary {
        name: "std".to_string(),
        nodes,
    }
}
