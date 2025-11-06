use crate::types::IglooType;

use super::*;
use indexmap::IndexMap;
use std::collections::HashMap;

pub fn std_library() -> PenguinLibrary {
    let mut nodes = HashMap::new();

    // TODO
    //
    // On Device Attach
    // On Device Detach
    // On Device Register
    //
    // On Floe Attach
    //
    // Get All Component? (requires change in typing)
    //
    // Get Device/Entity Name?

    add_query_node(
        &mut nodes,
        "Get One Component",
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Get One".to_string()),
            desc: "Read the value of the first component that matches the conditions. A value will populate and the 'Ok' branch will execute if successful. If no valid componenet was found the 'No Result' branch will execute.".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Execute"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            outputs: IndexMap::from([
                (PenguinPinID::from_str("Ok"), PenguinPinDefn::named_flow()),
                (
                    PenguinPinID::from_str("No Result"),
                    PenguinPinDefn::named_flow(),
                ),
            ]),
            ..Default::default()
        },
        false,
        false,
    );

    add_query_node(
        &mut nodes,
        "Aggregate Components",
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Aggregate".to_string()),
            desc: "Read all components that match conditions, then apply the operation. A value will populate and the 'Ok' branch will execute if successful. If no valid components were found OR that component does not support the aggregation function, the 'No Result' branch will execute.".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Execute"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            outputs: IndexMap::from([
                (PenguinPinID::from_str("Ok"), PenguinPinDefn::named_flow()),
                (
                    PenguinPinID::from_str("No Result"),
                    PenguinPinDefn::named_flow(),
                ),
            ]),
            ..Default::default()
        },
        true,
        false,
    );

    add_query_node(
        &mut nodes,
        "Set Components",
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Set".to_string()),
            desc: "Sets a component on all applicable entities.".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Execute"),
                PenguinPinDefn::unnamed_flow(),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Done"),
                PenguinPinDefn::named_flow(),
            )]),
            ..Default::default()
        },
        false,
        true,
    );

    add_query_node(
        &mut nodes,
        "On Component Changed",
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("On Change".to_string()),
            desc: "Any time a component that matches the filters changes, a value will populate and the branch will be executed. Note that it will initially trigger for all applicable components.".to_string(),
            outputs: IndexMap::from([
                (PenguinPinID::from_str("Trigger"), PenguinPinDefn::unnamed_flow()),
            ]),
            ..Default::default()
        },
        false,
        false,
    );

    nodes.insert(
        "Comment".to_string(),
        PenguinNodeDefn {
            version: 1,
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Text,
                input_type: NodeInputType::Input,
                id: NodeInputFeatureID::from_str("Value"),
            }],
            ..Default::default()
        },
    );

    nodes.insert(
        "Section".to_string(),
        PenguinNodeDefn {
            version: 1,
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Text,
                input_type: NodeInputType::Input,
                id: NodeInputFeatureID::from_str("Title"),
            }],
            is_section: true,
            ..Default::default()
        },
    );

    nodes.insert(
        "Delay Seconds".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Delay".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("Execute"),
                    PenguinPinDefn::unnamed_flow(),
                ),
                (
                    PenguinPinID::from_str("Wait (seconds)"),
                    PenguinPinDefn::named_val(IglooType::Integer),
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
        "Delay Milliseconds".to_string(),
        PenguinNodeDefn {
            version: 1,
            title_bar: Some("Delay".to_string()),
            inputs: IndexMap::from([
                (
                    PenguinPinID::from_str("Execute"),
                    PenguinPinDefn::unnamed_flow(),
                ),
                (
                    PenguinPinID::from_str("Wait (ms)"),
                    PenguinPinDefn::named_val(IglooType::Integer),
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
                    PenguinPinDefn::named_val(IglooType::Text),
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
                    PenguinPinDefn::named_val(IglooType::Boolean),
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

    add_variadic_node(
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

    add_variadic_node(
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
                PenguinPinDefn::unnamed_val(IglooType::Text),
            )]),
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Text,
                input_type: NodeInputType::Input,
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
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Boolean,
                input_type: NodeInputType::Input,
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
                PenguinPinDefn::unnamed_val(IglooType::Integer),
            )]),
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Integer,
                input_type: NodeInputType::Input,
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
                PenguinPinDefn::unnamed_val(IglooType::Real),
            )]),
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Real,
                input_type: NodeInputType::Input,
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
                PenguinPinDefn::unnamed_val(IglooType::Color),
            )]),
            input_features: vec![NodeInputFeature {
                value_type: IglooType::Color,
                input_type: NodeInputType::Input,
                id: NodeInputFeatureID::from_str("value"),
            }],
            ..Default::default()
        },
    );

    add_variadic_node(
        &mut nodes,
        "And",
        PenguinNodeDefn {
            version: 1,
            icon: "AND".to_string(),
            icon_bg: true,
            desc: "Logical AND".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            ..Default::default()
        },
    );

    add_variadic_node(
        &mut nodes,
        "Or",
        PenguinNodeDefn {
            version: 1,
            icon: "OR".to_string(),
            icon_bg: true,
            desc: "Logical OR".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Boolean),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Boolean),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Boolean),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Boolean),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            ..Default::default()
        },
    );

    add_variadic_node(
        &mut nodes,
        "Add Integers",
        PenguinNodeDefn {
            version: 1,
            icon: "+".to_string(),
            icon_bg: true,
            desc: "Add integers".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
            )]),
            ..Default::default()
        },
    );

    add_variadic_node(
        &mut nodes,
        "Multiply Integers",
        PenguinNodeDefn {
            version: 1,
            icon: "*".to_string(),
            icon_bg: true,
            desc: "Multiply integers".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            ..Default::default()
        },
    );

    add_variadic_node(
        &mut nodes,
        "Add Reals",
        PenguinNodeDefn {
            version: 1,
            icon: "+".to_string(),
            icon_bg: true,
            desc: "Add reals".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(IglooType::Real),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Real),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Real),
            )]),
            ..Default::default()
        },
    );

    add_variadic_node(
        &mut nodes,
        "Multiply Reals",
        PenguinNodeDefn {
            version: 1,
            icon: "*".to_string(),
            icon_bg: true,
            desc: "Multiply reals".to_string(),
            inputs: IndexMap::from([(
                PenguinPinID::from_str("Input {{2..10}}"),
                PenguinPinDefn::unnamed_val(IglooType::Real),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Real),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Real),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::unnamed_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
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
                    PenguinPinDefn::named_val(IglooType::Color),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::named_val(IglooType::Color),
                ),
                (
                    PenguinPinID::from_str("Ratio"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Color),
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
                    PenguinPinDefn::named_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("G"),
                    PenguinPinDefn::named_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::named_val(IglooType::Integer),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Color),
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
                PenguinPinDefn::unnamed_val(IglooType::Color),
            )]),
            outputs: IndexMap::from([
                (
                    PenguinPinID::from_str("R"),
                    PenguinPinDefn::named_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("G"),
                    PenguinPinDefn::named_val(IglooType::Integer),
                ),
                (
                    PenguinPinID::from_str("B"),
                    PenguinPinDefn::named_val(IglooType::Integer),
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
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("L"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Color),
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
                PenguinPinDefn::unnamed_val(IglooType::Color),
            )]),
            outputs: IndexMap::from([
                (
                    PenguinPinID::from_str("H"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("L"),
                    PenguinPinDefn::named_val(IglooType::Real),
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
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("V"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Color),
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
                PenguinPinDefn::unnamed_val(IglooType::Color),
            )]),
            outputs: IndexMap::from([
                (
                    PenguinPinID::from_str("H"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("S"),
                    PenguinPinDefn::named_val(IglooType::Real),
                ),
                (
                    PenguinPinID::from_str("V"),
                    PenguinPinDefn::named_val(IglooType::Real),
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
                PenguinPinDefn::unnamed_val(IglooType::Text),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Integer),
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
                PenguinPinDefn::unnamed_val(IglooType::Text),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Text),
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
                PenguinPinDefn::unnamed_val(IglooType::Text),
            )]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Text),
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
                    PenguinPinDefn::named_val(IglooType::Text),
                ),
                (
                    PenguinPinID::from_str("Find"),
                    PenguinPinDefn::named_val(IglooType::Text),
                ),
                (
                    PenguinPinID::from_str("Replace"),
                    PenguinPinDefn::named_val(IglooType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Text),
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
                    PenguinPinDefn::named_val(IglooType::Text),
                ),
                (
                    PenguinPinID::from_str("Pattern"),
                    PenguinPinDefn::named_val(IglooType::Text),
                ),
            ]),
            outputs: IndexMap::from([(
                PenguinPinID::from_str("Output"),
                PenguinPinDefn::unnamed_val(IglooType::Boolean),
            )]),
            ..Default::default()
        },
    );

    add_cast_nodes(&mut nodes);

    add_reroute(&mut nodes, PenguinPinType::Flow);
    add_reroute(&mut nodes, PenguinPinType::Value(IglooType::Integer));
    add_reroute(&mut nodes, PenguinPinType::Value(IglooType::Real));
    add_reroute(&mut nodes, PenguinPinType::Value(IglooType::Text));
    add_reroute(&mut nodes, PenguinPinType::Value(IglooType::Boolean));
    add_reroute(&mut nodes, PenguinPinType::Value(IglooType::Color));

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

fn add_cast_nodes(nodes: &mut HashMap<String, PenguinNodeDefn>) {
    for from in IglooType::all() {
        for to in IglooType::all() {
            if let Some(cast_name) = from.cast_name(to) {
                nodes.insert(
                    cast_name,
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
        }
    }
}

fn add_query_node(
    nodes: &mut HashMap<String, PenguinNodeDefn>,
    base_name: &str,
    mut template: PenguinNodeDefn,
    is_aggregate: bool,
    is_setter: bool,
) {
    template.query_feature = Some(NodeQueryFeature {
        base: base_name.to_string(),
        is_aggregate,
    });

    // base
    nodes.insert(base_name.to_string(), template.clone());

    // variants
    for r#type in IglooType::all() {
        let mut node = template.clone();

        if is_setter {
            node.inputs.insert(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::named_val(r#type),
            );
        } else {
            node.outputs.insert(
                PenguinPinID::from_str("Value"),
                PenguinPinDefn::named_val(r#type),
            );
            node.outputs.insert(
                PenguinPinID::from_str("Device ID"),
                PenguinPinDefn::named_val(IglooType::Integer),
            );
            node.outputs.insert(
                PenguinPinID::from_str("Entity ID"),
                PenguinPinDefn::named_val(IglooType::Integer),
            );
        }

        node.hide_search = true;

        nodes.insert(format!("{base_name} {type}"), node);
    }
}

fn add_variadic_node(
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
