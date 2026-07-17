use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(InputManagerPlugin::<Action>::default())
        ;
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    Jump,
    Dash,
}

#[derive(Bundle)]
pub struct InputBundle {
    input_map: InputMap<Action>,
    action_state: ActionState<Action>,
}

impl Default for InputBundle {
    fn default() -> Self {
        #[cfg(not(target_os = "android"))]
        let input_map = InputMap::default()
            .with_dual_axis(Action::Move, VirtualDPad::wasd())
            .with(Action::Jump, KeyCode::Space)
            .with(Action::Dash, KeyCode::KeyE);

        #[cfg(target_os = "android")]
        let input_map = InputMap::default();

        InputBundle {
            input_map,
            action_state: ActionState::<Action>::default(),
        }
    }
}