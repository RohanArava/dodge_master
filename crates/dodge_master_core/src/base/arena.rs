use avian2d::prelude::*;
use bevy::prelude::*;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, arena_init);
    }
}

#[derive(Component)]
pub struct ArenaWall;

fn arena_init(mut commands: Commands, window_query: Query<&Window>) {
    let window_width = 1280.0;
    let window_height = 720.0;
    
    let half_width = window_width / 2.0;
    let half_height = window_height / 2.0;

    // Ground
    commands.spawn((
        ArenaWall,
        RigidBody::Static,
        Collider::rectangle(window_width, 50.0),
        Transform::from_xyz(0.0, -half_height, 0.0),
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(window_width, 50.0)),
            ..default()
        },
    ));

    // Left wall
    commands.spawn((
        ArenaWall,
        RigidBody::Static,
        Collider::rectangle(50.0, window_height),
        Transform::from_xyz(-half_width, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, window_height)),
            ..default()
        },
    ));

    // Right wall
    commands.spawn((
        ArenaWall,
        RigidBody::Static,
        Collider::rectangle(50.0, window_height),
        Transform::from_xyz(half_width, 0.0, 0.0),
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(50.0, window_height)),
            ..default()
        },
    ));
}