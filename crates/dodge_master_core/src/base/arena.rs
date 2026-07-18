use avian2d::prelude::*;
use bevy::prelude::*;

use crate::base::utils::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, arena_init);
    }
}

#[derive(Component)]
pub struct ArenaWall;

fn arena_init(mut commands: Commands, window_query: Query<&Window>) {
    let half_width = WINDOW_WIDTH / 2.0;
    let half_height = WINDOW_HEIGHT / 2.0;

    // Ground
    commands.spawn((
        ArenaWall,
        RigidBody::Static,
        Collider::rectangle(WINDOW_WIDTH, 50.0),
        Transform::from_xyz(0.0, -half_height, 0.0),
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, 50.0)),
            ..default()
        },
    ));

    // Left wall
    commands.spawn((
        ArenaWall,
        RigidBody::Static,
        Collider::rectangle(50.0, WINDOW_HEIGHT),
        Transform::from_xyz(-half_width, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, WINDOW_HEIGHT)),
            ..default()
        },
    ));

    // Right wall
    commands.spawn((
        ArenaWall,
        RigidBody::Static,
        Collider::rectangle(50.0, WINDOW_HEIGHT),
        Transform::from_xyz(half_width, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, WINDOW_HEIGHT)),
            ..default()
        },
    ));
}
