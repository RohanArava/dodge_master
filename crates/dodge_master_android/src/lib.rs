use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
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
    pub pointer: Option<PointerId>,
}

#[bevy_main]
fn main() {
    let mut app = dodge_master_core::get_app();

    app.add_systems(Startup, (setup_android_joystick, setup_action_buttons))
        .add_systems(PreUpdate, write_touch_move);

    app.run();
}

fn setup_android_joystick(mut commands: Commands) {
    let max_radius = 90.0;
    let zone_size = max_radius * 2.0;

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
                pointer: None,
            },
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
        .observe(
            |t: On<Pointer<Press>>,
             mut zones: Query<(&mut JoystickZone, &UiGlobalTransform, &ComputedNode)>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                let (Ok((mut zone, transform, computed)), Ok((mut node, mut knob))) =
                    (zones.single_mut(), knobs.single_mut())
                else {
                    return;
                };

                if zone.pointer.is_none() {
                    zone.pointer = Some(t.pointer_id);
                }

                let pointer_position = t.pointer_location.position;

                let zone_position = transform.translation * computed.inverse_scale_factor;

                let mut offset = pointer_position - zone_position;

                if offset.length() > zone.max_radius {
                    offset = offset.normalize() * zone.max_radius;
                }

                node.left = Val::Px(offset.x);
                node.top = Val::Px(offset.y);

                let mut value = offset / zone.max_radius;
                value.y *= -1.0;
                knob.value = value;
            },
        )
        .observe(
            |t: On<Pointer<Drag>>,
             mut zones: Query<(&mut JoystickZone, &UiGlobalTransform, &ComputedNode)>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                let (Ok((zone, transform, computed)), Ok((mut node, mut knob))) =
                    (zones.single_mut(), knobs.single_mut())
                else {
                    return;
                };
                if zone.pointer != Some(t.pointer_id) {
                    return;
                }

                let pointer_position = t.pointer_location.position;

                let zone_position = transform.translation * computed.inverse_scale_factor;

                let mut offset = pointer_position - zone_position;

                if offset.length() > zone.max_radius {
                    offset = offset.normalize() * zone.max_radius;
                }

                node.left = Val::Px(offset.x);
                node.top = Val::Px(offset.y);

                let mut value = offset / zone.max_radius;
                value.y *= -1.0;
                knob.value = value;
            },
        )
        .observe(
            |t: On<Pointer<DragEnd>>,
             mut zones: Query<&mut JoystickZone>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                release(&t.pointer_id, &mut zones, &mut knobs);
            },
        )
        .observe(
            |t: On<Pointer<Release>>,
             mut zones: Query<&mut JoystickZone>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                release(&t.pointer_id, &mut zones, &mut knobs);
            },
        )
        .observe(
            |t: On<Pointer<Cancel>>,
             mut zones: Query<&mut JoystickZone>,
             mut knobs: Query<(&mut Node, &mut JoystickKnob)>| {
                release(&t.pointer_id, &mut zones, &mut knobs);
            },
        );
}

fn release(
    id: &PointerId,
    zones: &mut Query<&mut JoystickZone>,
    knobs: &mut Query<(&mut Node, &mut JoystickKnob)>,
) {
    let Ok(mut zone) = zones.single_mut() else {
        return;
    };
    if zone.pointer != Some(*id) {
        return;
    }
    zone.pointer = None;
    if let Ok((mut node, mut knob)) = knobs.single_mut() {
        reset_stick(&mut node, &mut knob);
    }
}

fn reset_stick(knob: &mut Node, k: &mut JoystickKnob) {
    knob.left = Val::Px(0.0);
    knob.top = Val::Px(0.0);
    k.value = Vec2::ZERO;
}

fn write_touch_move(knob: Query<&JoystickKnob>, mut touch: ResMut<TouchMove>) {
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
