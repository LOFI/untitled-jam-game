mod player;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
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

use player::PlayerPlugin;

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
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PkvStore::new("LOFI", "untitled-jam-game"))
        .init_state::<GameState>()
        .add_audio_channel::<BackgroundMusic>()
        .add_audio_channel::<SoundFX>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LOFI Untitled Jam Game".to_string(),
                        resolution: (640., 480.).into(),
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
        .add_plugins(PlayerPlugin)
        .add_plugins((WorldInspectorPlugin::new(), EditorPlugin::default())) // Egui editors
        .add_systems(Startup, (spawn_camera, spawn_floor))
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_floor(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
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
