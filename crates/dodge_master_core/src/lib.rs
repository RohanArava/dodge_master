pub mod base;

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::base::character::CharacterPlugin;
use crate::base::input;
use crate::base::arena::ArenaPlugin;

pub fn get_app() -> App{
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Dodge Master".to_string(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(input::InputPlugin)
    .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
    .add_plugins(PhysicsDebugPlugin::default())
    .add_plugins(CharacterPlugin)
    .add_plugins(ArenaPlugin)
    .add_systems(Startup, setup_camera);

    return app;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d::default(),
    Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::AutoMin { 
                min_width: 1280.0, 
                min_height: 720.0, 
            },
            ..OrthographicProjection::default_3d()
        }),
));
    // commands.spawn((
    //     RigidBody::Static,
    //     Collider::rectangle(2000.0, 50.0),
    //     Transform::from_xyz(0.0, -300.0, 0.0),
    // ));
}
