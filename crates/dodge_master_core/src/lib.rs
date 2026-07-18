pub mod base;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::base::arena::ArenaPlugin;
use crate::base::attack::orchestrator::AttackOrchestratorPlugin;
use crate::base::character::CharacterPlugin;
use crate::base::input;
use crate::base::state::GamePlugin;

pub fn get_app() -> App {
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
    .add_plugins(AttackOrchestratorPlugin)
    .add_plugins(GamePlugin)
    .add_systems(Startup, setup_camera);

    return app;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::AutoMin {
                min_width: 1280.0,
                min_height: 720.0,
            },
            ..OrthographicProjection::default_3d()
        }),
    ));
}
