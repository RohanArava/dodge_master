use bevy::prelude::*;
use dodge_master_core::base::character::Character;
use dodge_master_core::base::input::Action;
use leafwing_input_manager::prelude::*;

#[derive(Component, Default)]
pub struct JoystickKnob {
    pub value: Vec2,
    pub max_radius: f32,
}

#[bevy_main]
fn main() {
    let mut app = dodge_master_core::get_app();

    app.add_systems(Startup, (setup_android_joystick, setup_action_buttons))
        .add_systems(PreUpdate, translate_joystick_to_leafwing);

    app.run();
}

fn setup_android_joystick(mut commands: Commands) {
    let max_radius = 90.0;

    commands
        .spawn((
            Node {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
                    JoystickKnob {
                        value: Vec2::ZERO,
                        max_radius,
                    },
                ))
                .observe(
                    |trigger: On<Pointer<Drag>>,
                     mut query: Query<(&mut Node, &mut JoystickKnob)>| {
                        let event = trigger.event();
                        if let Ok((mut node, mut knob)) = query.get_mut(trigger.event_target()) {
                            // `distance` is the total offset from where the drag started,
                            // not the per-frame delta. Using `delta` here caused the knob
                            // to snap back toward zero every frame instead of tracking
                            // the finger's actual position.
                            let drag_offset = event.distance;

                            if drag_offset.length() > knob.max_radius {
                                let constrained = drag_offset.normalize() * knob.max_radius;
                                node.left = Val::Px(constrained.x);
                                node.top = Val::Px(constrained.y);
                                knob.value = constrained / knob.max_radius;
                            } else {
                                node.left = Val::Px(drag_offset.x);
                                node.top = Val::Px(drag_offset.y);
                                knob.value = drag_offset / knob.max_radius;
                            }

                            knob.value.y *= -1.0;
                        }
                    },
                )
                .observe(
                    |trigger: On<Pointer<DragEnd>>,
                     mut query: Query<(&mut Node, &mut JoystickKnob)>| {
                        if let Ok((mut node, mut knob)) = query.get_mut(trigger.event_target()) {
                            node.left = Val::Px(0.0);
                            node.top = Val::Px(0.0);
                            knob.value = Vec2::ZERO;
                        }
                    },
                );
        });
}

fn translate_joystick_to_leafwing(
    joystick_query: Query<&JoystickKnob>,
    mut player_query: Query<&mut ActionState<Action>, With<Character>>,
) {
    if let Ok(knob) = joystick_query.single() {
        if let Ok(mut action_state) = player_query.single_mut() {
            action_state.set_axis_pair(&Action::Move, knob.value);
        }
    }
}

#[derive(Component)]
pub struct ActionButton(pub Action);

fn setup_action_buttons(mut commands: Commands) {
    spawn_action_button(&mut commands, Action::Jump, 50.0, 150.0, "JUMP");
    spawn_action_button(&mut commands, Action::Dash, 50.0, 60.0, "DASH");
}

fn spawn_action_button(
    commands: &mut Commands,
    action: Action,
    right: f32,
    bottom: f32,
    label: &str,
) {
    commands
        .spawn((
            Node {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(bottom),
                right: Val::Px(right),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BackgroundColor(Color::srgba(0.8, 0.8, 0.8, 0.4)),
            ActionButton(action),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        })
        .observe(
            |trigger: On<Pointer<Press>>,
             buttons: Query<&ActionButton>,
             mut player_query: Query<&mut ActionState<Action>, With<Character>>| {
                if let Ok(button) = buttons.get(trigger.event_target()) {
                    if let Ok(mut action_state) = player_query.single_mut() {
                        action_state.press(&button.0);
                    }
                }
            },
        )
        .observe(
            |trigger: On<Pointer<Release>>,
             buttons: Query<&ActionButton>,
             mut player_query: Query<&mut ActionState<Action>, With<Character>>| {
                if let Ok(button) = buttons.get(trigger.event_target()) {
                    if let Ok(mut action_state) = player_query.single_mut() {
                        action_state.release(&button.0);
                    }
                }
            },
        );
}
