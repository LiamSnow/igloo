use bevy::{prelude::*, text::TextBounds};
use bevy_prototype_lyon::prelude::*;

#[derive(Component)]
#[require(Position)]
#[require(Title)]
struct Node;

#[derive(Component, Default)]
#[require(Transform)]
struct Position(Vec2);

#[derive(Component, Default)]
struct Title(String);

pub fn spawn_nodes(mut commands: Commands) {
    let nodes = vec![("Node A", 0, 0), ("Node B", 640, 0)];

    for (title, x, y) in nodes {
        let width = 380.;
        let height = 250.;
        let node_size = Vec2::new(width, height);

        let body_shape = shapes::Rectangle {
            extents: node_size,
            origin: RectangleOrigin::Center,
            radii: Some(BorderRadii::single(16.0)),
        };
        let header_size = Vec2::new(width, 64.0);
        let header_shape = shapes::Rectangle {
            extents: header_size,
            origin: RectangleOrigin::Center,
            radii: Some(BorderRadii::top(16.0)),
        };

        let flow_pin_points = [
            Vec2::new(0.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.8, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ]
        .map(|x| x * 15.);
        let flow_pin = shapes::RoundedPolygon {
            points: flow_pin_points.into_iter().collect(),
            radius: 2.,
            closed: true,
        };

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        commands.spawn((
            ShapeBuilder::with(&body_shape)
                .fill(Color::srgb(0.25, 0.25, 0.25))
                .build(),
            Transform::from_xyz((x << 6) as f32, (y << 6) as f32, 0.0),
            children![
                (
                    // shadow
                    ShapeBuilder::with(&body_shape)
                        .fill(Color::srgb(0.15, 0.15, 0.15))
                        .build(),
                    Transform::from_xyz(6.0, -6.0, -0.1),
                ),
                (
                    // header
                    ShapeBuilder::with(&header_shape)
                        .fill(Color::srgb(0.6, 0.25, 0.25))
                        .build(),
                    Transform::from_xyz(0.0, half_height - 32.0, 0.1),
                ),
                (
                    Text2d::new(title),
                    TextFont {
                        font_size: 30.0,
                        ..Default::default()
                    },
                    TextBounds::from(header_size),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_xyz(0.0, half_height - 64.0 / 1.4, 0.2),
                ),
                (
                    // output pin
                    ShapeBuilder::with(&flow_pin)
                        .fill(Color::srgb(1.0, 1.0, 1.0))
                        .build(),
                    Transform::from_xyz(half_width - 36.0, half_height - 64.0 - 28.0, 0.1),
                ),
                (
                    // input pin
                    ShapeBuilder::with(&flow_pin)
                        .fill(Color::srgb(1.0, 1.0, 1.0))
                        .build(),
                    Transform::from_xyz(-half_width + 36.0, half_height - 64.0 - 28.0, 0.1)
                        .with_scale(Vec3::new(-1.0, 1.0, 1.0)), // flip horizontally
                ),
            ],
        ));
    }
}
