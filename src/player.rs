use std::time::Duration;

use crate::{
    animation::{AnimationIndices, AnimationTimer},
    WINDOW_LEFT_X,
};
use bevy::{asset::LoadedFolder, prelude::*, render::texture::ImageSampler};
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
        app.init_resource::<PlayerSpriteSheet>()
            .init_state::<PlayerState>()
            .add_systems(OnEnter(PlayerState::Setup), load_textures)
            .add_systems(OnExit(PlayerState::Setup), spawn_player)
            .add_systems(
                FixedUpdate,
                idle_animation.run_if(in_state(PlayerState::Idle)),
            )
            .add_systems(Update, (fall, movement))
            .add_systems(Update, check_textures.run_if(in_state(PlayerState::Setup)));
    }
}

#[derive(Resource, Default)]
struct PlayerSpriteFolder(Handle<LoadedFolder>);

#[derive(Resource)]
struct PlayerSpriteSheet(Handle<TextureAtlasLayout>);

impl FromWorld for PlayerSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas = TextureAtlasLayout::from_grid(
            Vec2::new(48.0, 48.0), // sprite tile size
            10,                    // columns
            1,                     // rows
            None,                  // No padding
            None,                  // No separation
        );

        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}

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

fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> (TextureAtlasLayout, Handle<Image>) {
    let mut texture_atlas_builder =
        TextureAtlasBuilder::default().padding(padding.unwrap_or_default());

    for handle in folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!("{:?} is not an image", handle.path().unwrap());
            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture) = texture_atlas_builder.finish().unwrap();
    let texture = textures.add(texture);

    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (texture_atlas_layout, texture)
}

fn create_sprite_from_atlas(
    commands: &mut Commands,
    translation: (f32, f32, f32),
    sprite_index: usize,
    atlas_handle: Handle<TextureAtlasLayout>,
    texture: Handle<Image>,
    frames: Option<AnimationIndices>,
) {
    let sprite_dimensions = Vec2::new(64.0, 64.0);
    let sprite_hitbox = Vec2::new(24.0, 24.0);

    commands.spawn((
        SpriteSheetBundle {
            sprite: Sprite {
                custom_size: Some(sprite_dimensions),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(translation.0, translation.1, translation.2),
                ..default()
            },
            texture,
            atlas: TextureAtlas {
                index: sprite_index,
                layout: atlas_handle,
            },
            ..default()
        },
        Player,
        Direction::Right,
        RigidBody::KinematicPositionBased,
        KinematicCharacterController::default(),
        Collider::cuboid(sprite_hitbox.x, sprite_hitbox.y),
        AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        AnimationIndices {
            first: frames.as_ref().map_or(0, |f| f.first),
            last: frames.as_ref().map_or(0, |f| f.last),
        },
    ));
}

fn spawn_player(
    mut commands: Commands,
    // player_sprite_folder: Res<PlayerSpriteFolder>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    // loaded_folders: Res<Assets<LoadedFolder>>,
    // mut textures: ResMut<Assets<Image>>,
) {
    // let loaded_folder = loaded_folders.get(&player_sprite_folder.0).unwrap();

    // let (texture_atlas, texture) = create_texture_atlas(loaded_folder, None, None, &mut textures);
    // let atlas_handle = texture_atlases.add(texture_atlas.clone());
    // let image_handle: Handle<Image> = asset_server
    // .get_handle("sprites/player/idle-48x48.png")
    // .unwrap();

    // let sprite_index = texture_atlas.get_texture_index(&image_handle).unwrap();

    let texture = asset_server.load("sprites/player/idle-48x48.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(48.0, 48.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 9 };
    // let translation = Vec3::ZERO;
    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            // transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn idle_animation(
    mut commands: Commands,
    player_sprite_folder: Res<PlayerSpriteFolder>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    query: Query<(Entity, &Transform), With<Player>>,
) {
    if query.is_empty() {
        return;
    }

    let loaded_folder = loaded_folders.get(&player_sprite_folder.0).unwrap();
    let (entity, transform) = query.single();

    let (texture_atlas, texture) = create_texture_atlas(loaded_folder, None, None, &mut textures);
    let atlas_handle = texture_atlases.add(texture_atlas.clone());
    let image_handle: Handle<Image> = asset_server
        .get_handle("sprites/player/idle-48x48.png")
        .unwrap();
    let sprite_index = texture_atlas.get_texture_index(&image_handle).unwrap();

    // create_sprite_from_atlas(
    //     &mut commands,
    //     transform.translation.into(),
    //     sprite_index,
    //     atlas_handle,
    //     texture,
    //     Some(AnimationIndices { first: 0, last: 9 }),
    // )
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
