use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_pan, camera_zoom));
    }
}

fn camera_zoom(mut mouse_wheel: EventReader<MouseWheel>, mut query: Query<&mut Projection>) {
    for event in mouse_wheel.read() {
        if let Ok(mut projection) = query.single_mut()
            && let Projection::Orthographic(projection) = projection.as_mut()
        {
            let zoom_fact = 1.0 - event.y * 0.003;
            projection.scale = (projection.scale * zoom_fact).clamp(0.1, 10.0);
        }
    }
}

fn camera_pan(
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
