use super::*;
use indexmap::IndexMap;
use std::collections::HashMap;

pub fn std_library() -> PenguinLibrary {
    let mut nodes = HashMap::new();

    nodes.insert(
        "comment".to_string(),
        NodeDefn {
            version: 1,
            title: "Comment".to_string(),
            desc: "Comment".to_string(),
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: PenguinType::Text,
                id: InputID::from_str("Value"),
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "on_start".to_string(),
        NodeDefn {
            version: 1,
            title: "On Start".to_string(),
            style: NodeStyle::normal(""),
            desc: "Triggers when program starts".to_string(),
            outputs: IndexMap::from([(PinID::from_str("On Trigger"), PinDefn::unnamed_flow())]),
            ..Default::default()
        },
    );

    nodes.insert(
        "print".to_string(),
        NodeDefn {
            version: 1,
            title: "Print".to_string(),
            style: NodeStyle::normal(""),
            desc: "Prints text to console".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("Execute"), PinDefn::unnamed_flow()),
                (
                    PinID::from_str("Message"),
                    PinDefn::named_val(PenguinType::Text),
                ),
            ]),
            outputs: IndexMap::from([(PinID::from_str("Done"), PinDefn::unnamed_flow())]),
            ..Default::default()
        },
    );

    nodes.insert(
        "branch".to_string(),
        NodeDefn {
            version: 1,
            title: "Branch".to_string(),
            style: NodeStyle::normal(""),
            desc: "Conditionally split flow".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("Call"), PinDefn::unnamed_flow()),
                (
                    PinID::from_str("Condition"),
                    PinDefn::named_val(PenguinType::Bool),
                ),
            ]),
            outputs: IndexMap::from([
                (PinID::from_str("True"), PinDefn::named_flow()),
                (PinID::from_str("False"), PinDefn::named_flow()),
            ]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "merge",
        "Merge",
        "Execute once all branches have completed",
        NodeStyle::normal(""),
        (2, 10),
        "Input_",
        PinDefnType::Flow,
        true,
        "Output_",
        PinDefn::unnamed_flow(),
    );

    add_variadic(
        &mut nodes,
        "either",
        "Either",
        "Execute if either branch triggers",
        NodeStyle::normal(""),
        (2, 10),
        "Input_",
        PinDefnType::Flow,
        true,
        "Output_",
        PinDefn::unnamed_flow(),
    );

    nodes.insert(
        "const_text".to_string(),
        NodeDefn {
            version: 1,
            title: "Text Constant".to_string(),
            desc: "Text constant".to_string(),
            outputs: IndexMap::from([(
                PinID::from_str("Value"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: PenguinType::Text,
                id: InputID::from_str("value"),
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_bool".to_string(),
        NodeDefn {
            version: 1,
            title: "Boolean Constant".to_string(),
            desc: "Boolean constant".to_string(),
            outputs: IndexMap::from([(
                PinID::from_str("Value"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: PenguinType::Bool,
                id: InputID::from_str("value"),
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_int".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Constant".to_string(),
            desc: "Integer constant".to_string(),
            outputs: IndexMap::from([(
                PinID::from_str("Value"),
                PinDefn::unnamed_val(PenguinType::Int),
            )]),
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: PenguinType::Int,
                id: InputID::from_str("value"),
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_real".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Constant".to_string(),
            desc: "Real number constant".to_string(),
            outputs: IndexMap::from([(
                PinID::from_str("Value"),
                PinDefn::unnamed_val(PenguinType::Real),
            )]),
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: PenguinType::Real,
                id: InputID::from_str("value"),
            })],
            ..Default::default()
        },
    );

    nodes.insert(
        "const_color".to_string(),
        NodeDefn {
            version: 1,
            title: "Color Constant".to_string(),
            desc: "Color constant".to_string(),
            outputs: IndexMap::from([(
                PinID::from_str("Value"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            cfg: vec![NodeConfig::Input(InputConfig {
                r#type: PenguinType::Color,
                id: InputID::from_str("value"),
            })],
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "and",
        "AND",
        "Logical AND",
        NodeStyle::background("AND"),
        (2, 10),
        "Input_",
        PinDefnType::Value(PenguinType::Bool),
        true,
        "Output_",
        PinDefn::unnamed_val(PenguinType::Bool),
    );

    add_variadic(
        &mut nodes,
        "or",
        "OR",
        "Logical OR",
        NodeStyle::background("OR"),
        (2, 10),
        "Input_",
        PinDefnType::Value(PenguinType::Bool),
        true,
        "Output_",
        PinDefn::unnamed_val(PenguinType::Bool),
    );

    nodes.insert(
        "not".to_string(),
        NodeDefn {
            version: 1,
            title: "NOT".to_string(),
            style: NodeStyle::background("NOT"),
            desc: "Logical NOT".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "xor".to_string(),
        NodeDefn {
            version: 1,
            title: "XOR".to_string(),
            style: NodeStyle::background("XOR"),
            desc: "Logical XOR".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Bool),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Bool),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "bool_equal".to_string(),
        NodeDefn {
            version: 1,
            title: "Boolean Equal".to_string(),
            style: NodeStyle::background("=="),
            desc: "Boolean equality".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Bool),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Bool),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "int_add",
        "Add Integers",
        "Add integers",
        NodeStyle::background("+"),
        (2, 10),
        "Input_",
        PinDefnType::Value(PenguinType::Int),
        true,
        "Output",
        PinDefn::unnamed_val(PenguinType::Int),
    );

    nodes.insert(
        "int_sub".to_string(),
        NodeDefn {
            version: 1,
            title: "Subtract Integers".to_string(),
            style: NodeStyle::background("-"),
            desc: "Subtract integers".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "int_mul",
        "Multiply Integers",
        "Multiply integers",
        NodeStyle::background("*"),
        (2, 10),
        "Input_",
        PinDefnType::Value(PenguinType::Int),
        true,
        "Output",
        PinDefn::unnamed_val(PenguinType::Int),
    );

    nodes.insert(
        "int_div".to_string(),
        NodeDefn {
            version: 1,
            title: "Divide Integers".to_string(),
            style: NodeStyle::background("/"),
            desc: "Divide integers".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_mod".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Modulo".to_string(),
            style: NodeStyle::background("MOD"),
            desc: "Integer remainder".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_lt".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Less Than".to_string(),
            style: NodeStyle::background("<"),
            desc: "A < B".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_gt".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Greater Than".to_string(),
            style: NodeStyle::background(">"),
            desc: "A > B".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_lte".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Less Than or Equal".to_string(),
            style: NodeStyle::background("<="),
            desc: "A <= B".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_gte".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Greater Than or Equal".to_string(),
            style: NodeStyle::background(">="),
            desc: "A >= B".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_eq".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Equal".to_string(),
            style: NodeStyle::background("=="),
            desc: "A == B".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "int_neq".to_string(),
        NodeDefn {
            version: 1,
            title: "Integer Not Equal".to_string(),
            style: NodeStyle::background("!="),
            desc: "A != B".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::unnamed_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::unnamed_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "real_add",
        "Add Reals",
        "Add reals",
        NodeStyle::background("+"),
        (2, 10),
        "Input_",
        PinDefnType::Value(PenguinType::Real),
        true,
        "Output",
        PinDefn::unnamed_val(PenguinType::Real),
    );

    nodes.insert(
        "real_sub".to_string(),
        NodeDefn {
            version: 1,
            title: "Subtract Reals".to_string(),
            style: NodeStyle::background("-"),
            desc: "Subtract reals".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Real),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "real_mul",
        "Multiply Reals",
        "Multiply reals",
        NodeStyle::background("*"),
        (2, 10),
        "Input_",
        PinDefnType::Value(PenguinType::Real),
        true,
        "Output",
        PinDefn::unnamed_val(PenguinType::Real),
    );

    nodes.insert(
        "real_div".to_string(),
        NodeDefn {
            version: 1,
            title: "Divide Reals".to_string(),
            style: NodeStyle::background("/"),
            desc: "Divide reals".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Real),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "real_lt".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Less Than".to_string(),
            style: NodeStyle::background("<"),
            desc: "A < B".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "real_gt".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Greater Than".to_string(),
            style: NodeStyle::background(">"),
            desc: "A > B".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "real_lte".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Less Than or Equal".to_string(),
            style: NodeStyle::background("<="),
            desc: "A <= B".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "real_gte".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Greater Than or Equal".to_string(),
            style: NodeStyle::background(">="),
            desc: "A >= B".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "real_eq".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Equal".to_string(),
            style: NodeStyle::background("=="),
            desc: "A == B".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "real_neq".to_string(),
        NodeDefn {
            version: 1,
            title: "Real Not Equal".to_string(),
            style: NodeStyle::background("!="),
            desc: "A != B".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("A"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PinID::from_str("B"),
                    PinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_mix".to_string(),
        NodeDefn {
            version: 1,
            title: "Mix".to_string(),
            style: NodeStyle::normal(""),
            desc: "Blend two colors".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("A"), PinDefn::named_val(PenguinType::Color)),
                (PinID::from_str("B"), PinDefn::named_val(PenguinType::Color)),
                (
                    PinID::from_str("Ratio"),
                    PinDefn::named_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_rgb".to_string(),
        NodeDefn {
            version: 1,
            title: "From RGB".to_string(),
            style: NodeStyle::normal(""),
            desc: "Create from RGB".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("R"), PinDefn::named_val(PenguinType::Int)),
                (PinID::from_str("G"), PinDefn::named_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::named_val(PenguinType::Int)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_rgb".to_string(),
        NodeDefn {
            version: 1,
            title: "To RGB".to_string(),
            style: NodeStyle::normal(""),
            desc: "Extract RGB".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            outputs: IndexMap::from([
                (PinID::from_str("R"), PinDefn::named_val(PenguinType::Int)),
                (PinID::from_str("G"), PinDefn::named_val(PenguinType::Int)),
                (PinID::from_str("B"), PinDefn::named_val(PenguinType::Int)),
            ]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_hsl".to_string(),
        NodeDefn {
            version: 1,
            title: "From HSL".to_string(),
            style: NodeStyle::normal(""),
            desc: "Create from HSL".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("H"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("S"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("L"), PinDefn::named_val(PenguinType::Real)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_hsl".to_string(),
        NodeDefn {
            version: 1,
            title: "To HSL".to_string(),
            style: NodeStyle::normal(""),
            desc: "Extract HSL".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            outputs: IndexMap::from([
                (PinID::from_str("H"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("S"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("L"), PinDefn::named_val(PenguinType::Real)),
            ]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_from_hsv".to_string(),
        NodeDefn {
            version: 1,
            title: "From HSV".to_string(),
            style: NodeStyle::normal(""),
            desc: "Create from HSV".to_string(),
            inputs: IndexMap::from([
                (PinID::from_str("H"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("S"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("V"), PinDefn::named_val(PenguinType::Real)),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "color_to_hsv".to_string(),
        NodeDefn {
            version: 1,
            title: "To HSV".to_string(),
            style: NodeStyle::normal(""),
            desc: "Extract HSV".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Color),
            )]),
            outputs: IndexMap::from([
                (PinID::from_str("H"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("S"), PinDefn::named_val(PenguinType::Real)),
                (PinID::from_str("V"), PinDefn::named_val(PenguinType::Real)),
            ]),
            ..Default::default()
        },
    );

    nodes.insert(
        "text_length".to_string(),
        NodeDefn {
            version: 1,
            title: "Length".to_string(),
            style: NodeStyle::normal(""),
            desc: "Text length".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "text_to_upper".to_string(),
        NodeDefn {
            version: 1,
            title: "To Uppercase".to_string(),
            style: NodeStyle::background("→"),
            desc: "Convert to uppercase".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "text_to_lower".to_string(),
        NodeDefn {
            version: 1,
            title: "To Lowercase".to_string(),
            style: NodeStyle::background("→"),
            desc: "Convert to lowercase".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "text_replace".to_string(),
        NodeDefn {
            version: 1,
            title: "Replace".to_string(),
            style: NodeStyle::normal(""),
            desc: "Replace text".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("Text"),
                    PinDefn::named_val(PenguinType::Text),
                ),
                (
                    PinID::from_str("Find"),
                    PinDefn::named_val(PenguinType::Text),
                ),
                (
                    PinID::from_str("Replace"),
                    PinDefn::named_val(PenguinType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Text),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "text_regex_match".to_string(),
        NodeDefn {
            version: 1,
            title: "Regex Match".to_string(),
            style: NodeStyle::normal(""),
            desc: "Match regex pattern".to_string(),
            inputs: IndexMap::from([
                (
                    PinID::from_str("Text"),
                    PinDefn::named_val(PenguinType::Text),
                ),
                (
                    PinID::from_str("Pattern"),
                    PinDefn::named_val(PenguinType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    add_cast(&mut nodes, PenguinType::Text, PenguinType::Int);
    add_cast(&mut nodes, PenguinType::Real, PenguinType::Int);
    add_cast(&mut nodes, PenguinType::Bool, PenguinType::Int);

    add_cast(&mut nodes, PenguinType::Text, PenguinType::Real);
    add_cast(&mut nodes, PenguinType::Int, PenguinType::Real);
    add_cast(&mut nodes, PenguinType::Bool, PenguinType::Real);

    add_cast(&mut nodes, PenguinType::Text, PenguinType::Bool);
    add_cast(&mut nodes, PenguinType::Int, PenguinType::Bool);
    add_cast(&mut nodes, PenguinType::Real, PenguinType::Bool);

    add_cast(&mut nodes, PenguinType::Int, PenguinType::Text);
    add_cast(&mut nodes, PenguinType::Real, PenguinType::Text);
    add_cast(&mut nodes, PenguinType::Bool, PenguinType::Text);
    add_cast(&mut nodes, PenguinType::Color, PenguinType::Text);

    PenguinLibrary {
        name: "std".to_string(),
        nodes,
    }
}

fn add_cast(nodes: &mut HashMap<String, NodeDefn>, from: PenguinType, to: PenguinType) {
    nodes.insert(
        from.cast_name(to).unwrap(),
        NodeDefn {
            version: 1,
            title: format!("Cast {from} to {to}"),
            style: NodeStyle::background("→"),
            desc: "Casting will TODO".to_string(),
            inputs: IndexMap::from([(
                PinID::from_str("Input"),
                PinDefn::unnamed(PinDefnType::Value(from)),
            )]),
            outputs: IndexMap::from([(
                PinID::from_str("Output"),
                PinDefn::unnamed(PinDefnType::Value(to)),
            )]),
            ..Default::default()
        },
    );
}

fn add_variadic(
    nodes: &mut HashMap<String, NodeDefn>,
    base_name: &str,
    title: &str,
    desc: &str,
    style: NodeStyle,
    range: (u8, u8),
    input_base_id: &str,
    input_type: PinDefnType,
    input_hide_name: bool,
    output_id: &str,
    output_defn: PinDefn,
) {
    let (min, max) = range;

    for count in min..=max {
        let name = format!("{}_{}", base_name, count);

        let mut inputs = IndexMap::new();
        for i in 0..count {
            let pin_id = PinID::from_str(&format!("{}{}", input_base_id, i));
            let pin_defn = if input_hide_name {
                PinDefn::unnamed(input_type.clone())
            } else {
                PinDefn::named(input_type.clone())
            };
            inputs.insert(pin_id, pin_defn);
        }

        let outputs = IndexMap::from([(PinID::from_str(output_id), output_defn.clone())]);

        let prev = (count > min).then(|| format!("{}_{}", base_name, count - 1));
        let next = (count < max).then(|| format!("{}_{}", base_name, count + 1));

        nodes.insert(
            name,
            NodeDefn {
                version: 1,
                title: title.to_string(),
                desc: desc.to_string(),
                style: style.clone(),
                inputs,
                outputs,
                cfg: vec![NodeConfig::Variadic(VariadicConfig { prev, next })],
            },
        );
    }
}
