use super::*;
use indexmap::IndexMap;
use std::collections::HashMap;

pub fn std_library() -> PenguinLibrary {
    let mut nodes = HashMap::new();

    nodes.insert(
        "Comment".to_string(),
        PenguinNodeDefn {
            version: 1,
            input_features: vec![NodeInputFeature {
                r#type: PenguinType::Text,
                id: NodeInputFeatureID::from_str("Value"),
            }],
            ..Default::default()
        },
    );

    nodes.insert(
        "On Start".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("On Start".to_string()),
            desc: "Triggers when program starts".to_string(),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("On Trigger"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Print".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Print".to_string()),
            desc: "Prints text to console".to_string(),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("Execute"),
                    PenguinPinDefn::unnamed_flow(),
                ),
                (
                    PenguinPinID::from_str("Message"),
                    PenguinPinDefn::named_val(PenguinType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Done"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Branch".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Branch".to_string()),
            desc: "Conditionally split flow".to_string(),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("Call"),
                    PenguinPinDefn::unnamed_flow(),
                ),
                (
                    PenguinPinID::from_str("Condition"),
                    PenguinPinDefn::named_val(PenguinType::Bool),
                ),
            ]),
            outputs: IndexMap::from([
                (PenguinPinID::from_str("True"), PenguinPinDefn::named_flow()),
                (
                    PenguinPinID::from_str("False"),
                    PenguinPinDefn::named_flow(),
                ),
            ]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Merge",
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Merge".to_string()),
            desc: "Execute once all branches have completed".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Either",
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Either".to_string()),
            desc: "Execute if either branch triggers".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Text Constant".to_string(),
        PenguinNodeDefn {
            version: 1,
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            input_features: vec![NodeInputFeature {
                r#type: PenguinType::Text,
                id: NodeInputFeatureID::from_str("value"),
            }],
            ..Default::default()
        },
    );

    nodes.insert(
        "Boolean Constant".to_string(),
        PenguinNodeDefn {
            version: 1,
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            input_features: vec![NodeInputFeature {
                r#type: PenguinType::Bool,
                id: NodeInputFeatureID::from_str("value"),
            }],
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Constant".to_string(),
        PenguinNodeDefn {
            version: 1,
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            input_features: vec![NodeInputFeature {
                r#type: PenguinType::Int,
                id: NodeInputFeatureID::from_str("value"),
            }],
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Constant".to_string(),
        PenguinNodeDefn {
            version: 1,
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            input_features: vec![NodeInputFeature {
                r#type: PenguinType::Real,
                id: NodeInputFeatureID::from_str("value"),
            }],
            ..Default::default()
        },
    );

    nodes.insert(
        "Color Constant".to_string(),
        PenguinNodeDefn {
            version: 1,
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            input_features: vec![NodeInputFeature {
                r#type: PenguinType::Color,
                id: NodeInputFeatureID::from_str("value"),
            }],
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "And",
        PenguinNodeDefn {
            version: 1,
            icon: "AND".to_string(),
            icon_bg: true,
            desc: "Logical AND".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Or",
        PenguinNodeDefn {
            version: 1,
            icon: "OR".to_string(),
            icon_bg: true,
            desc: "Logical OR".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Not Boolean".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "NOT".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Xor Booleans".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "XOR".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Bool),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Bool),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Booleans Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "==".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Bool),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Bool),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Add Integers",
        PenguinNodeDefn {
            version: 1,
            icon: "+".to_string(),
            icon_bg: true,
            desc: "Add integers".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Subtract Integers".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "-".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Multiply Integers",
        PenguinNodeDefn {
            version: 1,
            icon: "*".to_string(),
            icon_bg: true,
            desc: "Multiply integers".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Divide Integers".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "/".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Modulo".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "MOD".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Less Than".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "<".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Greater Than".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: ">".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Less Than or Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "<=".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Greater Than or Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: ">=".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "==".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Integer Not Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "!=".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Add Reals",
        PenguinNodeDefn {
            version: 1,
            icon: "+".to_string(),
            icon_bg: true,
            desc: "Add reals".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Subtract Reals".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "-".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            ..Default::default()
        },
    );

    add_variadic(
        &mut nodes,
        "Multiply Reals",
        PenguinNodeDefn {
            version: 1,
            icon: "*".to_string(),
            icon_bg: true,
            desc: "Multiply reals".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Divide Reals".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "/".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Real),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Less Than".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "<".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Greater Than".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: ">".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Less Than or Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "<=".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Greater Than or Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: ">=".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "==".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Real Not Equal".to_string(),
        PenguinNodeDefn {
            version: 1,
            icon: "!=".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color Mix".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Mix".to_string()),
            desc: "Blend two colors".to_string(),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("A"),
                    PenguinPinDefn::named_val(PenguinType::Color),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::named_val(PenguinType::Color),
                ),
                (
                    PenguinPinID::from_str("Ratio"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color from RGB".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("From RGB".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("R"),
                    PenguinPinDefn::named_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("G"),
                    PenguinPinDefn::named_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::named_val(PenguinType::Int),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color to RGB".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("To RGB".to_string()),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            outputs: IndexMap::from([
                (
                    PenguinPinID::from_str("R"),
                    PenguinPinDefn::named_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("G"),
                    PenguinPinDefn::named_val(PenguinType::Int),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::named_val(PenguinType::Int),
                ),
            ]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color from HSL".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("From HSL".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("H"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("L"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color to HSL".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("To HSL".to_string()),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            outputs: IndexMap::from([
                (
                    PenguinPinID::from_str("H"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("L"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
            ]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color from HSV".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("From HSV".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("H"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("V"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Color to HSV".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("To HSV".to_string()),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Color),
            )]),
            outputs: IndexMap::from([
                (
                    PenguinPinID::from_str("H"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
                (
                    PenguinPinID::from_str("V"),
                    PenguinPinDefn::named_val(PenguinType::Real),
                ),
            ]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Text Length".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Length".to_string()),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Int),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Text to Uppercase".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("To Uppercase".to_string()),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Text to Lowercase".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("To Lowercase".to_string()),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Text Replace".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Replace".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("Text"),
                    PenguinPinDefn::named_val(PenguinType::Text),
                ),
                (
                    PenguinPinID::from_str("Find"),
                    PenguinPinDefn::named_val(PenguinType::Text),
                ),
                (
                    PenguinPinID::from_str("Replace"),
                    PenguinPinDefn::named_val(PenguinType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Text),
            )]),
            ..Default::default()
        },
    );

    nodes.insert(
        "Text Regex Match".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Regex Match".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("Text"),
                    PenguinPinDefn::named_val(PenguinType::Text),
                ),
                (
                    PenguinPinID::from_str("Pattern"),
                    PenguinPinDefn::named_val(PenguinType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(PenguinType::Bool),
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

    add_reroute(&mut nodes, PenguinPinType::Flow);
    add_reroute(&mut nodes, PenguinPinType::Value(PenguinType::Int));
    add_reroute(&mut nodes, PenguinPinType::Value(PenguinType::Real));
    add_reroute(&mut nodes, PenguinPinType::Value(PenguinType::Text));
    add_reroute(&mut nodes, PenguinPinType::Value(PenguinType::Bool));
    add_reroute(&mut nodes, PenguinPinType::Value(PenguinType::Color));

    PenguinLibrary { nodes }
}

fn add_reroute(nodes: &mut HashMap<String, PenguinNodeDefn>, pin_type: PenguinPinType) {
    nodes.insert(
        format!("Reroute {pin_type}"),
        PenguinNodeDefn {
            version: 1,
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed(pin_type),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed(pin_type),
            )]),
            hide_search: true,
            is_reroute: true,
            ..Default::default()
        },
    );
}

fn add_cast(nodes: &mut HashMap<String, PenguinNodeDefn>, from: PenguinType, to: PenguinType) {
    nodes.insert(
        from.cast_name(to).unwrap(),
        PenguinNodeDefn {
            version: 1,
            icon: "→".to_string(),
            icon_bg: true,
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input"),
                PenguinPinDefn::unnamed(PenguinPinType::Value(from)),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed(PenguinPinType::Value(to)),
            )]),
            ..Default::default()
        },
    );
}

fn add_variadic(
    nodes: &mut HashMap<String, PenguinNodeDefn>,
    base_name: &str,
    template: PenguinNodeDefn,
) {
    let mut variadic_info = None;
    let mut variadic_key = None;

    for (key, _) in &template.inputs {
        let key_str = key.0.to_string();
        if let Some(start) = key_str.find("{{")
            && let Some(end) = key_str.find("}}")
        {
            let pattern = &key_str[start + 2..end];
            if let Some((min_str, max_str)) = pattern.split_once("..")
                && let (Ok(min), Ok(max)) = (min_str.parse::<u8>(), max_str.parse::<u8>())
            {
                variadic_info = Some((min, max, key_str[..start].to_string()));
                variadic_key = Some(key.clone());
                break;
            }
        }
    }

    let Some((min, max, input_base_id)) = variadic_info else {
        panic!("Invalid variadic config for {base_name}")
    };

    let Some(variadic_key) = variadic_key else {
        return;
    };

    let input_defn = template.inputs.get(&variadic_key).unwrap().clone();

    for count in min..=max {
        let name = format!("{} {}", base_name, count);

        let mut inputs = IndexMap::new();
        for i in 0..count {
            let pin_id = PenguinPinID::from_str(&format!("{}{}", input_base_id, i));
            inputs.insert(pin_id, input_defn.clone());
        }

        let prev = (count > min).then(|| format!("{} {}", base_name, count - 1));
        let next = (count < max).then(|| format!("{} {}", base_name, count + 1));

        let mut node_defn = template.clone();
        node_defn.inputs = inputs;
        node_defn.variadic_feature = Some(NodeVariadicFeature { prev, next });
        node_defn.hide_search = count != min;

        nodes.insert(name, node_defn);
    }
}
