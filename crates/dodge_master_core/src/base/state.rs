use avian2d::prelude::*;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .add_systems(Update, resume_hitstop);
    }
}

#[derive(Resource, Default)]
pub struct GameState {
    pub hit_stop_timer: Timer,
}

fn resume_hitstop(
    mut game_state: ResMut<GameState>,
    real_time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    game_state.hit_stop_timer.tick(real_time.delta());

    if game_state.hit_stop_timer.is_finished() {
        if virtual_time.is_paused() {
            virtual_time.unpause();
        }
        if physics_time.is_paused() {
            physics_time.unpause();
        }
    }
}
