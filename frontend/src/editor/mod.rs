use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use leptos_bevy_canvas::prelude::*;

use crate::editor::camera::{CameraPlugin, MainCamera};

mod camera;
pub mod model;
mod node;

use bevy::ecs::event::Event;

#[derive(Event)]
pub struct TextEvent {
    pub text: String,
}

pub fn set_text(mut event_reader: EventReader<TextEvent>) {
    for _event in event_reader.read() {
        // do something with the event
    }
}

pub fn init_bevy_app(text_receiver: BevyEventReceiver<TextEvent>) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy_canvas".into()),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(ShapePlugin)
    .add_plugins(CameraPlugin)
    .import_event_from_leptos(text_receiver)
    .add_systems(Update, set_text)
    .add_systems(Startup, (spawn_camera, node::spawn_nodes));

    app
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}
