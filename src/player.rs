use crate::animation::{AnimationIndices, AnimationTimer};
use crate::boulder::Boulder;
use crate::{DistanceTraveled, GameState, PlayerInputEvent};
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy::{asset::LoadedFolder, prelude::*};
use bevy_rapier2d::prelude::*;

#[derive(Clone, Component, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum PlayerState {
    #[default]
    Setup,
    Idle,
    Walk,
    Push,
    Hurt,
    Fall,
}

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

#[derive(Component)]
struct FatigueMarker;

#[derive(Component, Default, Reflect)]
pub struct Fatigue(f32);

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerState>()
            .register_type::<Fatigue>()
            .add_systems(OnEnter(PlayerState::Setup), load_textures)
            .add_systems(
                OnExit(GameState::MainMenu),
                (
                    start,
                    spawn_player,
                    setup_fatigue_marker.after(spawn_player),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    fall,
                    hurt,
                    movement.after(fall),
                    rotate,
                    push_boulder.after(movement),
                    update_sprite_direction,
                    update_fatigue_marker,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    check_textures.run_if(in_state(PlayerState::Setup)),
                    fall_animation.run_if(in_state(PlayerState::Fall)),
                    idle_animation.run_if(in_state(PlayerState::Idle)),
                    walk_animation.run_if(in_state(PlayerState::Walk)),
                    push_animation.run_if(in_state(PlayerState::Push)),
                    hurt_animation.run_if(in_state(PlayerState::Hurt)),
                    update_direction,
                    // log_transitions,
                    update_fatigue,
                ),
            );
    }
}

fn start(mut next_state: ResMut<NextState<PlayerState>>) {
    next_state.set(PlayerState::Setup);
}

#[derive(Resource, Default)]
struct PlayerSpriteFolder(Handle<LoadedFolder>);

fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // load multiple, individual sprites from a directory
    commands.insert_resource(PlayerSpriteFolder(
        asset_server.load_folder("sprites/player"),
    ));
}

fn check_textures(
    mut next_state: ResMut<NextState<PlayerState>>,
    player_sprite_folder: Res<PlayerSpriteFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&player_sprite_folder.0) {
            next_state.set(PlayerState::Idle);
        }
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture: Handle<Image> = asset_server.load("sprites/player/push-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };
    let translation = Vec3::new(-50., 0., 3.);

    commands.spawn((
        SpriteSheetBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..default()
            },
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            transform: Transform::from_translation(translation),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player,
        Direction::Right,
        RigidBody::Dynamic,
        KinematicCharacterController {
            // Don’t allow climbing slopes larger than 60 degrees.
            max_slope_climb_angle: 60.0_f32.to_radians(),
            // Automatically slide down on slopes smaller than 45 degrees.
            min_slope_slide_angle: 45.0_f32.to_radians(),
            snap_to_ground: Some(CharacterLength::Absolute(0.5)),
            slide: true,
            ..default()
        },
        Collider::cuboid(12.0, 24.0),
        AdditionalMassProperties::Mass(68.), // 150 lbs in kg
        Fatigue::default(),
        // For "tumbling" when fatigued
        ExternalForce {
            force: Vec2::new(-10.0, 10.0),
            torque: 0.0,
        },
        Damping {
            linear_damping: 1.0,
            angular_damping: 0.7,
        },
    ));
}

fn idle_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, output) = query.single();

    let texture: Handle<Image> = asset_server.load("sprites/player/idle-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };

    if output.desired_translation.x == 0.0 && output.grounded {
        commands
            .entity(entity)
            .insert(texture)
            .insert(texture_atlas_layout)
            .insert(animation_indices);
    }
}

fn walk_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, output) = query.single();

    let texture: Handle<Image> = asset_server.load("sprites/player/walk-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 8, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 7 };

    if output.desired_translation.x != 0.0 && output.grounded {
        commands
            .entity(entity)
            .insert(texture)
            .insert(texture_atlas_layout)
            .insert(animation_indices);
    }
}

fn push_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<(Entity, &KinematicCharacterControllerOutput), With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, output) = query.single();

    let texture: Handle<Image> = asset_server.load("sprites/player/push-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };

    if output.desired_translation.x != 0.0 && output.grounded {
        commands
            .entity(entity)
            .insert(texture)
            .insert(texture_atlas_layout)
            .insert(animation_indices);
    }
}

fn hurt_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<Entity, With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let entity = query.single();

    let texture: Handle<Image> = asset_server.load("sprites/player/hurt-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 4, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 3 };

    commands
        .entity(entity)
        .insert(texture)
        .insert(texture_atlas_layout)
        .insert(animation_indices);
}

fn fall_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<Entity, With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let entity = query.single();

    let texture: Handle<Image> = asset_server.load("sprites/player/jumping-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 3, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    commands
        .entity(entity)
        .insert(texture)
        .insert(texture_atlas_layout)
        .insert(animation_indices);
}

fn fall(time: Res<Time>, mut query: Query<&mut KinematicCharacterController>) {
    if query.is_empty() {
        return;
    }

    let mut player = query.single_mut();
    let movement = time.delta().as_secs_f32() * -100.0;
    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn rotate(
    mut query: Query<(&mut Transform, &KinematicCharacterControllerOutput)>,
    rapier_context: Res<RapierContext>,
) {
    if query.is_empty() {
        return;
    }

    let (mut transform, output) = query.single_mut();
    let ray_pos = Vec2::new(transform.translation.x, transform.translation.y);
    let ray_dir = Vec2::new(0.0, -1.0);
    let max_toi = 4.;
    let solid = true;
    let filter = QueryFilter::default();

    if let Some((_, intersection)) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_toi, solid, filter)
    {
        let hit_normal = intersection.normal;

        let target_angle = hit_normal.y.atan2(hit_normal.x);
        let smooth_angle = transform
            .rotation
            .lerp(Quat::from_rotation_z(target_angle), 0.1);
        if output.grounded {
            transform.rotation = smooth_angle;
        }
    }
}

fn movement(
    time: Res<Time>,
    mut events: EventReader<PlayerInputEvent>,
    mut query: Query<(&Transform, &mut KinematicCharacterController)>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut distance_traveled: ResMut<DistanceTraveled>,
) {
    if query.is_empty() {
        return;
    }

    let (_transform, mut player) = query.single_mut();
    let mut movement = 0.0;

    for event in events.read() {
        match event {
            PlayerInputEvent::MoveRight => {
                movement += time.delta_seconds() * 75.0;
                next_state.set(PlayerState::Walk);
            }
            PlayerInputEvent::MoveLeft => {
                movement -= time.delta_seconds() * 75.0;
                next_state.set(PlayerState::Walk);
            }
            PlayerInputEvent::Idle => {
                next_state.set(PlayerState::Idle);
            }
        }
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.0)),
    }
}

fn push_boulder(
    query: Query<&Transform, With<Player>>,
    boulder_query: Query<&Transform, With<Boulder>>,
    mut next_state: ResMut<NextState<PlayerState>>,
    mut distance_traveled: ResMut<DistanceTraveled>
) {
    if query.is_empty() || boulder_query.is_empty() {
        return;
    }

    let player_transform = query.single();
    let boulder_transform = boulder_query.single();

    let boulder_circle = BoundingCircle::new(boulder_transform.translation.truncate(), 64.0);
    let player_rect = Aabb2d::new(
        player_transform.translation.truncate(),
        Vec2::new(24.0, 24.0),
    );

    if boulder_circle.aabb_2d().intersects(&player_rect) {
        distance_traveled.0 += 1.;
        info!("{:2}", distance_traveled.0);
        next_state.set(PlayerState::Push);
    }
}

fn hurt(
    mut next_state: ResMut<NextState<PlayerState>>,
    mut player: Query<(&mut ExternalForce, &Fatigue), With<Player>>,
) {
    let (mut force, Fatigue(fatigue)) = match player.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    if *fatigue >= 99.0 {
        // FIXME: the backward motion will mean the player is likely "Falling"
        //   Need to hold the Hurt state for some period of time to avoid
        //   skipping over the animation entirely.
        next_state.set(PlayerState::Hurt);

        force.torque = 120.;
    } else {
        force.torque = 0.;
    }
}

fn update_direction(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput)>,
    mut next_state: ResMut<NextState<PlayerState>>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();

    if !output.grounded {
        next_state.set(PlayerState::Fall);
    }

    if output.desired_translation.x > 0. {
        commands.entity(player).insert(Direction::Right);
    } else if output.desired_translation.x < 0. {
        commands.entity(player).insert(Direction::Left);
    }
}

fn update_sprite_direction(mut query: Query<(&mut Sprite, &Direction)>) {
    if query.is_empty() {
        return;
    }

    let (mut sprite, direction) = query.single_mut();
    match direction {
        Direction::Right => {
            sprite.flip_x = false;
        }
        Direction::Left => {
            sprite.flip_x = true;
        }
    }
}

fn setup_fatigue_marker(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    player: Query<&Transform, With<Player>>,
) {
    let texture: Handle<Image> = asset_server.load("ui/fatigue.png");
    let layout = TextureAtlasLayout::from_grid(
        Vec2::new(46.0, 14.0), // tile size
        8,                     // columns
        1,                     // rows
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(layout);

    commands
        .spawn(SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: texture_atlas_handle,
                index: 0,
            },
            texture,
            transform: Transform {
                translation: player.single().translation + Vec3::new(0.0, 32.0, 100.0),
                ..default()
            },
            ..default()
        })
        .insert(FatigueMarker);
}

fn update_fatigue_marker(
    mut commands: Commands,
    player: Query<(&Transform, &Fatigue), With<Player>>,
    mut entity: Query<(Entity, &mut TextureAtlas), With<FatigueMarker>>,
) {
    if entity.is_empty() || player.is_empty() {
        return;
    }

    let (transform, fatigue) = player.single();

    let (entity, mut atlas) = entity.single_mut();

    match fatigue.0.ceil() as usize {
        0..=15 => {
            atlas.index = 0;
        }
        16..=30 => {
            atlas.index = 1;
        }
        31..=45 => {
            atlas.index = 2;
        }
        46..=60 => {
            atlas.index = 4;
        }
        61..=75 => {
            atlas.index = 5;
        }
        76..=90 => {
            atlas.index = 6;
        }
        _ => {
            atlas.index = 7;
        }
    }

    commands.entity(entity).insert(Transform {
        translation: transform.translation + Vec3::new(0.0, 32.0, 100.0),
        ..default()
    });
}

fn log_transitions(mut transitions: EventReader<StateTransitionEvent<PlayerState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.before, transition.after
        );
    }
}

fn update_fatigue(
    time: Res<Time>,
    mut query: Query<&mut Fatigue, With<Player>>,
    next_state: Res<NextState<PlayerState>>,
) {
    let state = match next_state.0 {
        Some(state) => state,
        // Nothing to do without a state.
        None => return,
    };

    let mut fatigue = match query.get_single_mut() {
        Err(_) => return,
        Ok(fatigue) => fatigue,
    };

    let updated = match state {
        PlayerState::Push => fatigue.0 + 5.0 * time.delta_seconds(),
        _ => fatigue.0 - 25.0 * time.delta_seconds(),
    }
    .clamp(0.0, 100.0);

    fatigue.0 = updated;
}
