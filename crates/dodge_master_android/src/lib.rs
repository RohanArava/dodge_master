use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use dodge_master_core::base::character::Character;
use dodge_master_core::base::input::{Action, TouchMove};
use leafwing_input_manager::prelude::*;

#[derive(Component, Default)]
pub struct JoystickKnob {
    pub value: Vec2,
    pub max_radius: f32,
}

#[derive(Component)]
pub struct JoystickZone {
    pub max_radius: f32,
    pub active: bool,
}

#[bevy_main]
fn main() {
    let mut app = dodge_master_core::get_app();

    app.add_systems(Startup, (setup_android_joystick, setup_action_buttons))
        .add_systems(
            PreUpdate,
            (update_joystick_from_cursor, write_touch_move).chain(),
        );

    app.run();
}

fn setup_android_joystick(mut commands: Commands) {
    let max_radius = 90.0;
    let zone_size = max_radius * 2.0;

    // Keep the zone centered on the same point the original 70x70 ring was.
    let ring_center_left = 80.0 + 35.0;
    let ring_center_bottom = 60.0 + 35.0;
    let zone_left = ring_center_left - max_radius;
    let zone_bottom = ring_center_bottom - max_radius;

    commands
        .spawn((
            Node {
                width: Val::Px(zone_size),
                height: Val::Px(zone_size),
                position_type: PositionType::Absolute,
                bottom: Val::Px(zone_bottom),
                left: Val::Px(zone_left),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            JoystickZone {
                max_radius,
                active: false,
            },
            RelativeCursorPosition::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(70.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(3.0)),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((
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
                    ));
                });
        })
        .observe(|_trigger: On<Pointer<Press>>, mut zones: Query<&mut JoystickZone>| {
            if let Ok(mut zone) = zones.single_mut() {
                zone.active = true;
            }
        })
        .observe(
            |_trigger: On<Pointer<Release>>,
             mut zones: Query<&mut JoystickZone>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                if let Ok(mut zone) = zones.single_mut() {
                    zone.active = false;
                }
                if let Ok((mut node, mut knob)) = knobs.single_mut() {
                    node.left = Val::Px(0.0);
                    node.top = Val::Px(0.0);
                    knob.value = Vec2::ZERO;
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<DragEnd>>,
             mut zones: Query<&mut JoystickZone>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                if let Ok(mut zone) = zones.single_mut() {
                    zone.active = false;
                }
                if let Ok((mut node, mut knob)) = knobs.single_mut() {
                    node.left = Val::Px(0.0);
                    node.top = Val::Px(0.0);
                    knob.value = Vec2::ZERO;
                }
            },
        );
}

fn update_joystick_from_cursor(
    zone_query: Query<(&JoystickZone, &RelativeCursorPosition)>,
    mut knobs: Query<(&mut Node, &mut JoystickKnob)>,
) {
    let Ok((zone, rel_cursor)) = zone_query.single() else {
        return;
    };
    let Ok((mut node, mut knob)) = knobs.single_mut() else {
        return;
    };

    if !zone.active {
        return;
    }

    let Some(normalized) = rel_cursor.normalized else {
        return;
    };

    // normalized: (0,0) at the zone's center, (±0.5, ±0.5) at its corners.
    let size = Vec2::splat(zone.max_radius * 2.0);
    let mut offset = normalized * size;

    if offset.length() > zone.max_radius {
        offset = offset.normalize() * zone.max_radius;
    }

    node.left = Val::Px(offset.x);
    node.top = Val::Px(offset.y);

    let mut value = offset / zone.max_radius;
    value.y *= -1.0;
    knob.value = value;
}

fn write_touch_move(
    knob: Query<&JoystickKnob>,
    mut touch: ResMut<TouchMove>,
) {
    let value = knob.single().map(|k| k.value).unwrap_or(Vec2::ZERO);
    if touch.0 != value {
        touch.0 = value;
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
            |trigger: On<Pointer<Enter>>,
             buttons: Query<&ActionButton>,
             mut player_query: Query<&mut ActionState<Action>, With<Character>>| {
                if let Ok(button) = buttons.get(trigger.event_target()) {
                    if let Ok(mut action_state) = player_query.single_mut() {
                        info!("button press");
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
        )
        .observe(
            |trigger: On<Pointer<Leave>>,
             buttons: Query<&ActionButton>,
             mut player_query: Query<&mut ActionState<Action>, With<Character>>| {
                if let Ok(button) = buttons.get(trigger.event_target()) {
                    if let Ok(mut action_state) = player_query.single_mut() {
                        action_state.release(&button.0);
                    }
                }
            },
        )
        .observe(
            |trigger: On<Pointer<Cancel>>,
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
