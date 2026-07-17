use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::input::{Action, InputBundle};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterStats>()
        .add_systems(Startup, lenny_init)
        .add_systems(Update, character_inputs)
            .add_systems(
                FixedUpdate,
                character_movement,
            );
    }
}

#[derive(Component)]
pub struct Character;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CharacterStats {
    pub speed: f32,
    pub jump_velocity: f32,
    pub second_jump_velocity: f32,
    pub double_jump_enabled: bool,
    pub gravity: f32,
    pub dash_speed: f32,
    pub dash_duration: f32,
    pub dash_cooldown: f32,
    pub dash_enabled: bool,
    pub input_buffer: i8,
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            speed: 500.0,
            jump_velocity: 1000.0,
            second_jump_velocity: 300.0,
            double_jump_enabled: true,
            gravity: 2800.0,
            dash_speed: 1800.0,
            dash_duration: 0.15,
            dash_cooldown: 0.5,
            dash_enabled: true,
            input_buffer: 8
        }
    }
}

#[derive(Component)]
pub struct CharacterState {
    pub velocity: Vec2,
    pub jumps_remaining: u8,
    pub facing: f32,
    pub grounded: bool,
    pub dash_timer: Timer,
    pub dash_cooldown_timer: Timer,
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            jumps_remaining: 0,
            facing: 1.0,
            grounded: false,
            dash_timer: Timer::from_seconds(0.0, TimerMode::Once),
            dash_cooldown_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct InputState {
    pub jump_pressed: i8,
    pub dash_pressed: i8,
    pub jump_just_released: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            jump_pressed: -1,
            dash_pressed: -1,
            jump_just_released: false
        }
    }
}

#[derive(Bundle)]
pub struct CharacterBundle {
    marker: Character,
    stats: CharacterStats,
    state: CharacterState,
    input_state: InputState,
    rigid_body: RigidBody,
    collider: Collider,
    transform: Transform,
    input: InputBundle,
}

impl CharacterBundle {
    pub fn new(position: Vec2) -> Self {
        let radius = 15.0;
        Self {
            marker: Character,
            stats: CharacterStats::default(),
            state: CharacterState::default(),
            input_state: InputState::default(),
            rigid_body: RigidBody::Kinematic,
            collider: Collider::circle(radius),
            transform: Transform::from_translation(position.extend(0.0)),
            input: InputBundle::default(),
        }
    }
}

fn lenny_init(mut commands: Commands) {
    commands.spawn((
        CharacterBundle::new(Vec2::new(0.0, 200.0)),
        Sprite {
            color: Color::srgb(0.9, 0.3, 0.3),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
    ));
}

fn character_inputs(
    mut query: Query<(&ActionState<Action>, &mut InputState, &CharacterStats), With<Character>>
) {
    for (action_state, mut input_state, stats) in &mut query {
        if action_state.just_pressed(&Action::Jump) {input_state.jump_pressed = stats.input_buffer;}
        if action_state.just_pressed(&Action::Dash) {input_state.dash_pressed = stats.input_buffer;}
        if action_state.just_released(&Action::Jump) {input_state.jump_just_released = true;}
    }
}

fn character_movement(
    time: Res<Time>,
    mut commands: Commands,
    move_and_slide: MoveAndSlide,
    mut query: Query<
        (Entity, &Collider, &Position, &Rotation, &ActionState<Action>, &mut CharacterState, &mut InputState, &CharacterStats),
        With<Character>,
    >,
) {
    for (entity, collider, position, rotation, action_state, mut state, mut input_state, stats) in &mut query {

        // tick timers
        state.dash_timer.tick(time.delta());
        state.dash_cooldown_timer.tick(time.delta());

        if input_state.dash_pressed >= 0 {input_state.dash_pressed -= 1;}
        if input_state.jump_pressed >= 0 {input_state.jump_pressed -= 1;}

        // without this downward velocity keeps on increasing due to gravity
        if state.grounded {
            state.jumps_remaining = if stats.double_jump_enabled {2} else {1};
            if state.velocity.y < 0.0 {
                state.velocity.y = 0.0;
            }
        }

        // Base direction and velocity
        let direction = action_state.axis_pair(&Action::Move).x;
        if direction != 0.0 {
            state.facing = direction.signum();
        }

        state.velocity.x = f32::copysign((direction.abs() >= 0.3) as i32 as f32, direction) * stats.speed;
        state.velocity.y -= stats.gravity * time.delta_secs();

        // Dash processing
        if input_state.dash_pressed >= 0
            && state.dash_timer.is_finished()
            && state.dash_cooldown_timer.is_finished()
            && stats.dash_enabled
        {
            input_state.dash_pressed = 0;
            state.dash_timer = Timer::from_seconds(stats.dash_duration, TimerMode::Once);
            state.dash_cooldown_timer = Timer::from_seconds(stats.dash_cooldown, TimerMode::Once);
        }

        if !state.dash_timer.is_finished() {
            state.velocity.x = state.facing * stats.dash_speed;
            state.velocity.y = 0.0;
        }

        // If jump is pressed give the character upward velocity
        if input_state.jump_pressed >= 0 && state.jumps_remaining > 0 {
            let is_second_jump = stats.double_jump_enabled && state.jumps_remaining==0;
            state.velocity.y = if is_second_jump {stats.second_jump_velocity} else {stats.jump_velocity};

            state.jumps_remaining -= 1;
            input_state.jump_pressed = 0;
        } 

        // If jump is released before max height (most of it), cancel jump (mostly)
        if input_state.jump_just_released && state.velocity.y > 10.0 {
            state.velocity.y = 10.0;
        }
        input_state.jump_just_released = false;

        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let mut touched_floor = false;

        let output = move_and_slide.move_and_slide(
            collider,
            position.0,
            rotation.as_radians(),
            state.velocity,
            time.delta(),
            &MoveAndSlideConfig::default(),
            &filter,
            |hit| {
                if hit.normal.y > 0.7 {
                    touched_floor = true;
                }
                MoveAndSlideHitResponse::Accept
            },
        );

        commands.entity(entity).insert(Position(output.position));
        state.velocity = output.projected_velocity;
        state.grounded = touched_floor;
    }
}