use crate::animation::{AnimationIndices, AnimationTimer};
use bevy::{asset::LoadedFolder, prelude::*};
use bevy_rapier2d::prelude::*;

#[derive(Clone, Component, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum PlayerState {
    #[default]
    Setup,
    Idle,
    Walk,
    Run,
    Push,
    Dead,
}

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PlayerState>()
            .add_systems(OnEnter(PlayerState::Setup), load_textures)
            .add_systems(OnExit(PlayerState::Setup), spawn_player)
            .add_systems(FixedUpdate, (fall, movement, update_sprite_direction))
            .add_systems(
                Update,
                (
                    check_textures.run_if(in_state(PlayerState::Setup)),
                    idle_animation.run_if(in_state(PlayerState::Idle)),
                    update_direction,
                ),
            );
    }
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
    let texture: Handle<Image> = asset_server
        .get_handle("sprites/player/push-48x48.png")
        .unwrap();
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };
    let translation = Vec3::ZERO;

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
        RigidBody::KinematicPositionBased,
        KinematicCharacterController::default(),
        Collider::cuboid(24.0, 24.0),
    ));
}

fn idle_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<(Entity, &Transform), With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, transform) = query.single();

    let texture: Handle<Image> = asset_server
        .get_handle("sprites/player/idle-48x48.png")
        .unwrap();
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlases.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };
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

fn movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut KinematicCharacterController>,
) {
    if query.is_empty() {
        return;
    }

    let mut player = query.single_mut();
    let mut movement = 0.0;

    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        movement += time.delta_seconds() * 100.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        movement -= time.delta_seconds() * 100.0;
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.0)),
    }
}

fn update_direction(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();

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
