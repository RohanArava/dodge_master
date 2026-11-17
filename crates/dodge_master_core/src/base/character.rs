use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::input::{Action, InputBundle, TouchMove};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CharacterStats>()
            .add_systems(Startup, lenny_init)
            .add_systems(Update, (character_inputs, animate_sprite))
            .add_systems(FixedUpdate, character_movement);
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

#[derive(Component)]
pub struct Hurtbox;

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            speed: 500.0,
            jump_velocity: 1200.0,
            second_jump_velocity: 1000.0,
            double_jump_enabled: true,
            gravity: 2800.0,
            dash_speed: 1200.0,
            dash_duration: 0.2,
            dash_cooldown: 0.8,
            dash_enabled: true,
            input_buffer: 8,
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
    pub lives: i8,
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
            lives: 5,
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
            jump_just_released: false,
        }
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

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
    hurt_box: Hurtbox,
    cee: CollisionEventsEnabled,
    sprite: Sprite,
    animation_indices: AnimationIndices,
    animation_timer: AnimationTimer,
}

impl Default for CharacterBundle {
    fn default() -> Self {
        let position = vec2(0.0, 200.0);
        let height = 84.0;
        let width = 60.0;
        Self {
            marker: Character,
            stats: CharacterStats::default(),
            state: CharacterState::default(),
            input_state: InputState::default(),
            rigid_body: RigidBody::Kinematic,
            collider: Collider::rectangle(width, height),
            transform: Transform::from_translation(position.extend(0.0)),
            input: InputBundle::default(),
            hurt_box: Hurtbox,
            cee: CollisionEventsEnabled,
            sprite: Sprite {
                color: Color::srgb(0.9, 0.3, 0.3),
                custom_size: Some(Vec2::splat(30.0)),
                ..default()
            },
            animation_indices: AnimationIndices { first: 0, last: 0 },
            animation_timer: AnimationTimer(Timer::from_seconds(0.0, TimerMode::Repeating)),
        }
    }
}

fn lenny_init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("behe_spritesheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(640), 4, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 5, last: 5 };

    let art_min = Vec2::new(60.0, 40.0); // left, top
    let art_max = Vec2::new(520.0, 600.0); // right, bottom (feet)
    let art_rect = Rect::from_corners(art_min, art_max);

    let scale = 0.2;
    let render_size = art_rect.size() * scale;

    commands.spawn((CharacterBundle {
        sprite: Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 5,
            }),
            custom_size: Some(render_size),
            rect: Some(art_rect),
            ..Default::default()
        },
        animation_indices: animation_indices,
        animation_timer: AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ..Default::default()
    },));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

fn character_inputs(
    mut query: Query<(&ActionState<Action>, &mut InputState, &CharacterStats), With<Character>>,
) {
    for (action_state, mut input_state, stats) in &mut query {
        if action_state.just_pressed(&Action::Jump) {
            input_state.jump_pressed = stats.input_buffer;
        }
        if action_state.just_pressed(&Action::Dash) {
            input_state.dash_pressed = stats.input_buffer;
        }
        if action_state.just_released(&Action::Jump) {
            input_state.jump_just_released = true;
        }
    }
}

fn character_movement(
    time: Res<Time>,
    mut commands: Commands,
    move_and_slide: MoveAndSlide,
    touch: Res<TouchMove>,
    mut query: Query<
        (
            Entity,
            &Collider,
            &Position,
            &Rotation,
            &ActionState<Action>,
            &mut CharacterState,
            &mut InputState,
            &CharacterStats,
            &mut Sprite,
            &mut AnimationIndices,
        ),
        With<Character>,
    >,
) {
    for (
        entity,
        collider,
        position,
        rotation,
        action_state,
        mut state,
        mut input_state,
        stats,
        mut sprite,
        mut animation_indices,
    ) in &mut query
    {
        // tick timers
        state.dash_timer.tick(time.delta());
        state.dash_cooldown_timer.tick(time.delta());

        if input_state.dash_pressed >= 0 {
            input_state.dash_pressed -= 1;
        }
        if input_state.jump_pressed >= 0 {
            input_state.jump_pressed -= 1;
        }

        // without this downward velocity keeps on increasing due to gravity
        if state.grounded {
            state.jumps_remaining = if stats.double_jump_enabled { 2 } else { 1 };
            if state.velocity.y < 0.0 {
                state.velocity.y = 0.0;
            }
        }

        // Base direction and velocity
        let direction = (action_state.axis_pair(&Action::Move) + touch.0).x;
        if direction != 0.0 {
            state.facing = direction.signum();
        }

        let velocity_direction = f32::copysign((direction.abs() >= 0.10) as i32 as f32, direction);

        state.velocity.x = velocity_direction * stats.speed;
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
            let is_second_jump = stats.double_jump_enabled && state.jumps_remaining == 1;
            state.velocity.y = if is_second_jump {
                stats.second_jump_velocity
            } else {
                stats.jump_velocity
            };

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

        sprite.flip_x = state.facing == 1.0;

        if velocity_direction != 0.0 {
            animation_indices.first = 0;
            animation_indices.last = 4;

            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index > 4 {
                    atlas.index = 0
                }
            }
        } else {
            animation_indices.first = 5;
            animation_indices.last = 5;

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = 5
            }
        }
    }
}
