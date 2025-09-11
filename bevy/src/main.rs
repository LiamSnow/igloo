use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};
use rand::random;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_pan, camera_zoom))
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color = Color::hsl(240., random::<f32>() * 0.3, random::<f32>() * 0.3);
            commands.spawn((
                Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.),
            ));
        }
    }
}

fn camera_zoom(mut mouse_wheel: EventReader<MouseWheel>, mut query: Query<&mut Projection>) {}

fn camera_pan(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&Projection, &mut Transform), With<MainCamera>>,
) {
    if !mouse_buttons.pressed(MouseButton::Right) {
        return;
    }

    for event in mouse_motion.read() {
        if let Ok((projection, mut transform)) = query.single_mut() {
            let scale = match projection {
                Projection::Orthographic(p) => p.scale,
                _ => continue,
            };

            transform.translation.x -= event.delta.x * scale;
            transform.translation.y += event.delta.y * scale;
        }
    }
}
