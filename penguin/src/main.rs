use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use camera::{CameraPlugin, MainCamera};

mod camera;
pub mod model;
mod node;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#bevy".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ShapePlugin)
        .add_plugins(CameraPlugin)
        .add_systems(Startup, (spawn_camera, node::spawn_nodes))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}
