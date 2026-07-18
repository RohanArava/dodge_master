use rand::RngExt;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::base::{
    character::{Character, CharacterState, Hurtbox},
    state::GameState,
    utils::{WINDOW_HEIGHT, WINDOW_WIDTH},
};

pub struct AttackOrchestratorPlugin;

impl Plugin for AttackOrchestratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, attack_init)
            .add_systems(
                Update,
                (attack_orchestration, laser_system_system, laser_system),
            )
            .add_systems(FixedUpdate, hitbox_hurtbox_system);
    }
}

#[derive(Component)]
pub struct AttackState {
    pub attacks_timer: Timer,
}

impl Default for AttackState {
    fn default() -> Self {
        Self {
            attacks_timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

fn attack_init(mut commands: Commands) {
    commands.spawn(AttackState::default());
}

fn attack_orchestration(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<&mut AttackState>,
) {
    let attack_state = &mut query.single_mut().expect("multiple attack states detected");
    attack_state.attacks_timer.tick(time.delta());

    if attack_state.attacks_timer.just_finished() {
        commands.spawn(LaserSystem::from_epoch_and_count(4, 4));
    }
}

#[derive(PartialEq)]
pub enum LaserSystemState {
    NotStarted,
    ChildrenSpawned,
}

#[derive(Component)]
pub struct LaserSystem {
    pub count: i8,
    pub epochs: i8,
    pub lasers: Vec<Entity>,
    pub current_epochs: i8,
    pub state: LaserSystemState,
}

impl Default for LaserSystem {
    fn default() -> Self {
        Self {
            count: 4,
            epochs: 4,
            lasers: Vec::new(),
            current_epochs: 0,
            state: LaserSystemState::NotStarted,
        }
    }
}

impl LaserSystem {
    fn from_epoch_and_count(epochs: i8, count: i8) -> Self {
        Self {
            epochs: epochs,
            count: count,
            ..Default::default()
        }
    }
}
fn laser_system_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LaserSystem)>,
    character_query: Query<(&Position, &CharacterState), With<Character>>,
) {
    for (entity, mut laser_system) in &mut query {
        let (position, character_state) = character_query.single().expect("Multiple players found");
        let character_velocity = character_state.velocity;

        match laser_system.state {
            LaserSystemState::NotStarted => {
                if laser_system.current_epochs < laser_system.epochs {
                    for i in 0..laser_system.count {
                        let (start, end) = if i == 0 {
                            let start = random_top_point();
                            let predicted = position.0
                                + character_velocity * LaserBundle::get_anticipation_time();
                            let end = boundary_intersection(start, predicted);
                            (start, end)
                        } else {
                            (random_top_point(), random_bottom_point())
                        };

                        let id = commands.spawn(LaserBundle::from_points(start, end)).id();
                        laser_system.lasers.push(id);
                    }
                    laser_system.state = LaserSystemState::ChildrenSpawned;
                }
            }
            LaserSystemState::ChildrenSpawned => {
                if count_live_entities(&mut commands, &laser_system.lasers) == 0 {
                    laser_system.state = LaserSystemState::NotStarted;
                    laser_system.current_epochs += 1;
                    laser_system.lasers = Vec::new();
                    if laser_system.current_epochs == laser_system.epochs {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

fn count_live_entities(commands: &mut Commands, entities: &Vec<Entity>) -> i32 {
    let mut count = 0;
    for entity in entities {
        if commands.get_spawned_entity(*entity).is_ok() {
            count += 1;
        }
    }
    count
}

#[derive(PartialEq)]
pub enum LaserPhase {
    Anticipation,
    Firing,
    Ending,
}

#[derive(Component)]
pub struct Laser;

#[derive(Component)]
pub struct LaserCollider;

#[derive(Component)]
pub struct LaserState {
    pub phase: LaserPhase,
    pub timer: Timer,
    pub coords: (Vec2, Vec2),
    pub length: f32,
    pub angle: f32,
    pub midpoint: Vec2,
    pub max_width: f32,
    pub anticipation_time: f32,
    pub firing_time: f32,
    pub ending_time: f32,
}

impl LaserState {
    fn from_points(p1: Vec2, p2: Vec2) -> Self {
        let diff = p2 - p1;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x);
        let midpoint = (p1 + p2) / 2.0;

        Self {
            phase: LaserPhase::Anticipation,
            timer: Timer::from_seconds(LaserBundle::get_anticipation_time(), TimerMode::Once),
            coords: (p1, p2),
            length,
            angle,
            midpoint,
            max_width: 20.0,
            anticipation_time: LaserBundle::get_anticipation_time(),
            firing_time: 1.2,
            ending_time: 0.6,
        }
    }
}

#[derive(Bundle)]
pub struct LaserBundle {
    pub state: LaserState,
    pub marker: Laser,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl LaserBundle {
    pub fn from_points(p1: Vec2, p2: Vec2) -> Self {
        let state = LaserState::from_points(p1, p2);

        let transform = Transform {
            translation: state.midpoint.extend(0.0),
            rotation: Quat::from_rotation_z(state.angle),
            scale: Vec3::new(state.length, 0.0, 1.0),
        };

        let sprite = Sprite {
            color: Color::WHITE.with_alpha(0.6),
            custom_size: Some(Vec2::ONE),
            ..default()
        };

        Self {
            state,
            marker: Laser,
            sprite,
            transform,
            global_transform: GlobalTransform::default(),
        }
    }

    fn get_anticipation_time() -> f32 {
        0.4
    }
}

fn laser_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LaserState, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut state, mut transform, mut sprite) in &mut query {
        state.timer.tick(time.delta());

        if state.timer.just_finished() {
            match state.phase {
                LaserPhase::Anticipation => {
                    state.phase = LaserPhase::Firing;
                    state.timer = Timer::from_seconds(state.firing_time, TimerMode::Once);
                    commands.entity(entity).insert((
                        Collider::rectangle(1.0, 1.0),
                        Sensor,
                        Hitbox { damage: 1 },
                        CollisionEventsEnabled,
                    ));
                    transform.scale.y = state.max_width;
                    sprite.color.set_alpha(1.0);
                }
                LaserPhase::Firing => {
                    state.phase = LaserPhase::Ending;
                    state.timer = Timer::from_seconds(state.ending_time, TimerMode::Once);
                    commands
                        .entity(entity)
                        .remove::<(Collider, Sensor, Hitbox, CollisionEventsEnabled)>();
                }
                LaserPhase::Ending => {
                    commands.entity(entity).despawn();
                    continue;
                }
            }
        }

        let t = state.timer.elapsed_secs() / state.timer.duration().as_secs_f32();

        match state.phase {
            LaserPhase::Anticipation => {
                transform.scale.y = state.max_width * t;
            }
            LaserPhase::Firing => {}
            LaserPhase::Ending => {
                transform.scale.y = state.max_width * (1.0 - t);
                sprite.color.set_alpha(1.0 - t);
            }
        }
    }
}

#[derive(Component)]
pub struct Hitbox {
    pub damage: u8,
}

fn hitbox_hurtbox_system(
    mut collision_events: MessageReader<CollisionStart>,
    hitboxes: Query<&Hitbox>,
    mut hurtboxes: Query<&mut CharacterState, With<Hurtbox>>,
    mut game_state: ResMut<GameState>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    for CollisionStart {
        collider1: a,
        collider2: b,
        body1: _,
        body2: _,
    } in collision_events.read()
    {
        for (hit, hurt) in [(*a, *b), (*b, *a)] {
            if let Ok(hitbox) = hitboxes.get(hit) {
                if let Ok(mut character) = hurtboxes.get_mut(hurt) {
                    character.lives -= hitbox.damage as i8;
                    info!("Hurt! health: {}", character.lives);
                    game_state.hit_stop_timer = Timer::from_seconds(0.1, TimerMode::Once);
                    virtual_time.pause();
                    physics_time.pause();
                }
            }
        }
    }
}

fn random_top_point() -> Vec2 {
    let mut rng = rand::rng();
    let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
    Vec2::new(x, WINDOW_HEIGHT / 2.0)
}

fn random_bottom_point() -> Vec2 {
    let mut rng = rand::rng();
    let x = rng.random_range(-WINDOW_WIDTH / 2.0..WINDOW_WIDTH / 2.0);
    Vec2::new(x, -WINDOW_HEIGHT / 2.0)
}

fn random_boundary_point() -> Vec2 {
    let mut rng = rand::rng();
    let half_w = WINDOW_WIDTH / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0;
    match rng.random_range(0..4) {
        0 => Vec2::new(rng.random_range(-half_w..half_w), half_h),
        1 => Vec2::new(rng.random_range(-half_w..half_w), -half_h),
        2 => Vec2::new(-half_w, rng.random_range(-half_h..half_h)),
        _ => Vec2::new(half_w, rng.random_range(-half_h..half_h)),
    }
}

fn boundary_intersection(start: Vec2, through: Vec2) -> Vec2 {
    let dir = through - start;
    let half_w = WINDOW_WIDTH / 2.0;
    let half_h = WINDOW_HEIGHT / 2.0;

    if dir.length_squared() < 1e-6 {
        return random_boundary_point();
    }

    let mut candidates = Vec::new();
    if dir.x != 0.0 {
        candidates.push((half_w - start.x) / dir.x);
        candidates.push((-half_w - start.x) / dir.x);
    }
    if dir.y != 0.0 {
        candidates.push((half_h - start.y) / dir.y);
        candidates.push((-half_h - start.y) / dir.y);
    }

    let mut best_t: Option<f32> = None;
    for t in candidates {
        if t > 1e-4 {
            let p = start + dir * t;
            if p.x >= -half_w - 0.5
                && p.x <= half_w + 0.5
                && p.y >= -half_h - 0.5
                && p.y <= half_h + 0.5
                && best_t.map_or(true, |b| t < b)
            {
                best_t = Some(t);
            }
        }
    }

    match best_t {
        Some(t) => start + dir * t,
        None => random_boundary_point(),
    }
}
