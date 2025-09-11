use dioxus::{html::geometry::ElementPoint, prelude::*};
use dioxus_elements::{geometry::WheelDelta, input_data::MouseButton};

use crate::penguin::{
    coordinates::{CoordinateSystem, ScreenPoint, ScreenVector},
    model::{
        registry::{NodeDefn, NodeType, NodeVIODefn, ValueType},
        scene::{NodePinRef, WireKind},
    },
    node::NodeComponent,
};

use super::coordinates::WorldPoint;
use super::model::{registry::NodeRegistry, scene::Scene};

enum DragMode {
    None,
    Panning,
}

#[component]
pub fn Workspace() -> Element {
    let (registry, init_scene) = example_scene();

    let mut scene = use_signal(|| init_scene);
    let mut camera_pos = use_signal(|| WorldPoint::new(0.0, 0.0));
    let mut zoom = use_signal(|| 1.0);
    let mut viewport_size = use_signal(|| (800.0, 600.0));
    let mut drag_mode = use_signal(|| DragMode::None);

    let coord_system = use_memo(move || {
        let (width, height) = *viewport_size.read();
        let mut cs = CoordinateSystem::new(width, height);
        cs.set_camera_position(*camera_pos.read());
        cs.set_zoom(*zoom.read());
        cs
    });

    let onmousedown = move |event: MouseEvent| {
        event.prevent_default();

        // let coords = event.data().element_coordinates();
        // let world = coord_system
        //     .read()
        //     .screen_to_world(ScreenPoint::new(coords.x as f32, coords.y as f32));

        match event.data().trigger_button() {
            Some(MouseButton::Primary) => {}
            Some(MouseButton::Secondary) => {
                drag_mode.set(DragMode::Panning);
            }
            _ => {}
        }
    };

    let onmouseup = move |event: MouseEvent| match event.data().trigger_button() {
        Some(MouseButton::Primary) => {}
        Some(MouseButton::Secondary) => {
            drag_mode.set(DragMode::None);
        }
        _ => {}
    };

    // TODO change to global listener
    let mut last_mouse = use_signal(|| ElementPoint::new(0., 0.));
    let onmousemove = move |event: MouseEvent| {
        let coords = event.data().element_coordinates();
        let delta = coords - *last_mouse.read();
        let delta = ScreenVector::new(delta.x as f32, delta.y as f32);

        match *drag_mode.read() {
            DragMode::None => {}
            DragMode::Panning => {
                let world_delta = coord_system.read().screen_to_world_vec(delta);
                let new_pos = *camera_pos.read() - world_delta;
                camera_pos.set(new_pos);
            }
        }

        last_mouse.set(coords);
    };

    rsx! {
        div {
            style: "width: 100%; height: calc(100vh - 41px); background-color: #222; position: relative; overflow: hidden; margin: 0; padding: 0; box-sizing: border-box;",
            oncontextmenu: |e| e.prevent_default(),
            onmousedown,
            onmouseup,
            onmousemove,
            onwheel,
            div {
                style: format!(
                    "transform: translate({:.2}px, {:.2}px) scale({:.3});
                     transform-origin: 0 0;
                     position: absolute;
                     width: 100%;
                     height: 100%;
                     margin: 0;
                     padding: 0;",
                    -camera_pos.read().x * *zoom.read() + viewport_size.read().0 / 2.0,
                    -camera_pos.read().y * *zoom.read() + viewport_size.read().1 / 2.0,
                    *zoom.read()
                ),
                for (id, node) in &scene.read().nodes {
                    NodeComponent{
                        id: *id,
                        defn: registry.get(&node.defn).unwrap().clone(),
                        pos: node.pos
                    }
                }
            }
        }
    }
}

fn example_scene() -> (NodeRegistry, Scene) {
    let mut registry = NodeRegistry::new();

    let on_start_def = 1;
    let print_def = 2;
    let add_def = 3;
    let number_def = 4;

    registry.insert(
        on_start_def,
        NodeDefn {
            icon: "▶️".to_string(),
            title: "On Start".to_string(),
            desc: "Triggered when the program starts".to_string(),
            vio: NodeVIODefn {
                inp: vec![],
                out: vec![("value".to_string(), ValueType::Number)],
            },
            typ: NodeType::Trigger,
        },
    );

    registry.insert(
        print_def,
        NodeDefn {
            icon: "".to_string(),
            title: "Print".to_string(),
            desc: "Prints a value to console".to_string(),
            vio: NodeVIODefn {
                inp: vec![("message".to_string(), ValueType::String)],
                out: vec![],
            },
            typ: NodeType::Action,
        },
    );

    registry.insert(
        add_def,
        NodeDefn {
            icon: "".to_string(),
            title: "Add".to_string(),
            desc: "Adds two numbers".to_string(),
            vio: NodeVIODefn {
                inp: vec![
                    ("a".to_string(), ValueType::Number),
                    ("b".to_string(), ValueType::Number),
                ],
                out: vec![("result".to_string(), ValueType::Number)],
            },
            typ: NodeType::Function,
        },
    );

    registry.insert(
        number_def,
        NodeDefn {
            icon: "".to_string(),
            title: "Number".to_string(),
            desc: "A constant number value".to_string(),
            vio: NodeVIODefn {
                inp: vec![],
                out: vec![("value".to_string(), ValueType::Number)],
            },
            typ: NodeType::Constant,
        },
    );

    let mut scene = Scene::empty();

    let on_start = scene.add_node(on_start_def, WorldPoint::new(90.0, 90.0));
    let _ = scene.add_node(print_def, WorldPoint::new(390.0, 90.0));
    let _ = scene.add_node(add_def, WorldPoint::new(240.0, 240.0));
    let add = scene.add_node(add_def, WorldPoint::new(240.0, -100.0));
    let num1 = scene.add_node(number_def, WorldPoint::new(90.0, 300.0));
    let num2 = scene.add_node(number_def, WorldPoint::new(90.0, 390.0));
    let print = scene.add_node(print_def, WorldPoint::new(190.0, 90.0));

    scene.connect(
        WireKind::Flow,
        NodePinRef {
            node: on_start,
            pin: 0,
        },
        NodePinRef {
            node: print,
            pin: 0,
        },
    );

    scene.connect(
        WireKind::Value,
        NodePinRef { node: num1, pin: 0 },
        NodePinRef { node: add, pin: 0 },
    );

    scene.connect(
        WireKind::Value,
        NodePinRef { node: num2, pin: 0 },
        NodePinRef { node: add, pin: 1 },
    );

    (registry, scene)
}
