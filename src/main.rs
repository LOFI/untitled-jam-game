mod animation;
mod boulder;
mod player;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    render::camera::RenderTarget,
    window::Window,
};
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::prelude::*;
use bevy_pkv::PkvStore;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;

use animation::AnimationPlugin;
use boulder::BoulderPlugin;
use player::{Player, PlayerPlugin};

const WINDOW_WIDTH: f32 = 640.;
const WINDOW_HEIGHT: f32 = 480.;
const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.;

const COLOR_BACKGROUND: Color = Color::BLACK;
const COLOR_FLOOR: Color = Color::GREEN;
const COLOR_WALL: Color = Color::WHITE;

#[derive(Resource)]
struct BackgroundMusic;

#[derive(Resource)]
struct SoundFX;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
enum GameState {
    #[default]
    Startup,
    MainMenu,
    InGame,
    GameOver,
}

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never) // Makes WASM happy
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .insert_resource(PkvStore::new("LOFI", "untitled-jam-game"))
        .init_state::<GameState>()
        .add_audio_channel::<BackgroundMusic>()
        .add_audio_channel::<SoundFX>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LOFI Untitled Jam Game".to_string(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // keeps pixel art crisp
        )
        .add_plugins(AudioPlugin) // Kira audio
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((AnimationPlugin, BoulderPlugin, PlayerPlugin))
        .add_plugins((WorldInspectorPlugin::new(), EditorPlugin::default())) // Egui editors
        .add_systems(Startup, (spawn_camera, spawn_floor, spawn_walls))
        .add_systems(Update, move_camera)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct UICamera;

const UI_LAYER: RenderLayers = RenderLayers::layer(9);

fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: WINDOW_WIDTH as u32,
        height: WINDOW_HEIGHT as u32,
        ..default()
    };

    // This image serves as a canvas for the UI layer
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    canvas.resize(canvas_size);
    let image_handle = images.add(canvas);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        },
        UICamera,
        UI_LAYER,
    ));

    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn move_camera(
    mut query: Query<(&mut Transform, &MainCamera), Without<Player>>,
    player_query: Query<(&Transform, &Player), With<KinematicCharacterController>>,
) {
    if query.is_empty() || player_query.is_empty() {
        return;
    }

    let mut camera = query.single_mut();
    let transform = player_query.single().0;

    camera.0.translation.x = transform.translation.x;
}

fn spawn_floor(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_FLOOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., -240., 0.),
                scale: Vec3::new(640., 20., 1.),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));
}

fn spawn_walls(mut commands: Commands) {
    // Left wall
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_WALL,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(WINDOW_LEFT_X - 10., 0., 0.),
                scale: Vec3::new(20., WINDOW_HEIGHT, 1.),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));

    // Right wall
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_WALL,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(150., -200., 0.),
                scale: Vec3::new(20., WINDOW_HEIGHT, 1.),
                rotation: Quat::from_rotation_z(PI / 1.5),
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));
}
