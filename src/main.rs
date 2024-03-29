mod animation;
mod boulder;
mod camera;
mod ground;
mod player;

use bevy::asset::AssetMetaCheck;
use bevy::audio::{PlaybackMode, Volume};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::prelude::*;
use bevy_pkv::PkvStore;
use bevy_rapier2d::prelude::*;
use rand::seq::SliceRandom;

use animation::AnimationPlugin;
use boulder::BoulderPlugin;
use camera::{CameraPlugin, UI_LAYER};
use ground::GroundPlugin;
use player::PlayerPlugin;

pub const WINDOW_WIDTH: f32 = 640.;
pub const WINDOW_HEIGHT: f32 = 480.;
const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.;

const COLOR_BACKGROUND: Color = Color::BLACK;
const COLOR_WALL: Color = Color::WHITE;

#[derive(Resource)]
struct BackgroundMusic;

#[derive(Resource)]
struct SoundFX;

#[derive(Resource)]
struct DistanceTraveled(f32);

#[derive(Event)]
pub enum PlayerInputEvent {
    MoveLeft,
    MoveRight,
    Idle,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum GameState {
    #[default]
    Startup,
    MainMenu,
    InGame,
    Pause,
    GiveUp,
    Cleanup,
}

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never) // Makes WASM happy
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .insert_resource(DistanceTraveled(0.))
        .insert_resource(PkvStore::new("LOFI", "sisyphus-simulator"))
        .init_state::<GameState>()
        .add_audio_channel::<BackgroundMusic>()
        .add_audio_channel::<SoundFX>()
        .add_event::<PlayerInputEvent>()
        .add_plugins(EmbeddedAssetPlugin {
            mode: PluginMode::ReplaceDefault,
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sisyphus Simulator".to_string(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // keeps pixel art crisp
        )
        .add_plugins(AudioPlugin) // Kira audio
        .add_plugins(TilemapPlugin) // ECS Tilemap
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(64.),
            // RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((
            AnimationPlugin,
            BoulderPlugin,
            CameraPlugin,
            GroundPlugin,
            PlayerPlugin,
        ))
        // .add_plugins(WorldInspectorPlugin::new()) // Egui editor
        .add_systems(Startup, (setup_background_music, spawn_background))
        .add_systems(
            Update,
            (
                volume, movement, pause,
                // log_transitions,
            ),
        )
        .add_systems(OnEnter(GameState::Pause), setup_pause_menu)
        .add_systems(Update, pause_menu_system.run_if(in_state(GameState::Pause)))
        .add_systems(OnExit(GameState::Pause), cleanup_pause_menu)
        .add_systems(OnExit(GameState::MainMenu), spawn_wall)
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(OnEnter(GameState::GiveUp), setup_give_up_menu)
        .add_systems(
            Update,
            give_up_menu_system.run_if(in_state(GameState::GiveUp)),
        )
        .add_systems(OnExit(GameState::GiveUp), cleanup_give_up_menu)
        .add_systems(
            Update,
            main_menu_button_system.run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnEnter(GameState::Cleanup), cleanup)
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        .run();
}

fn movement(keyboard_input: Res<ButtonInput<KeyCode>>, mut events: EventWriter<PlayerInputEvent>) {
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        events.send(PlayerInputEvent::MoveLeft);
    } else if keyboard_input.pressed(KeyCode::ArrowRight) {
        events.send(PlayerInputEvent::MoveRight);
    } else {
        events.send(PlayerInputEvent::Idle);
    }
}

fn spawn_wall(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_WALL,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(WINDOW_LEFT_X - 10., 0., 0.),
                scale: Vec3::new(20., WINDOW_HEIGHT * 2., 0.),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));
}

#[derive(Component)]
struct TitleText;

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let title_font: Handle<Font> = asset_server.load("fonts/Kaph-Regular.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                top: Val::Px(-100.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Sisyphus Simulator".to_string(),
                    TextStyle {
                        font_size: 60.0,
                        color: Color::WHITE,
                        font: title_font,
                    },
                )
                .with_text_justify(JustifyText::Center),
                UI_LAYER,
                TitleText,
            ));
        });

    let font = asset_server.load("fonts/PeaberryMono.ttf");

    let text_style = TextStyle {
        color: Color::WHITE,
        font_size: 25.0,
        font,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(30.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                ..default()
            },
            UI_LAYER,
        ))
        .with_children(|parent| {
            parent
                .spawn((ButtonBundle {
                    background_color: Color::PURPLE.into(),
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(150.),
                        height: Val::Px(50.),
                        margin: UiRect {
                            top: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play".to_string(),
                        text_style.clone(),
                    ));
                });

            parent
                .spawn((ButtonBundle {
                    background_color: Color::PURPLE.into(),
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(150.),
                        height: Val::Px(50.),
                        margin: UiRect {
                            top: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Quit".to_string(),
                        text_style.clone(),
                    ));
                });

            parent.spawn((
                TextBundle::from_section(
                    "-/= to lower/raise volume\n0 to mute".to_string(),
                    text_style.clone(),
                )
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    margin: UiRect {
                        top: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                }),
                UI_LAYER,
            ));
        });
}

fn cleanup_main_menu(
    mut commands: Commands,
    interaction_query: Query<Entity, With<Button>>,
    text_query: Query<Entity, With<Text>>,
) {
    for entity in &text_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &interaction_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn main_menu_button_system(
    mut state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(GameState::InGame);
    }

    for (interaction, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if text.sections[0].value == "Play" {
                    state.set(GameState::InGame);
                } else if text.sections[0].value == "Quit" {
                    std::process::exit(0);
                }
            }
            Interaction::Hovered => {
                text.sections[0].style.font_size = 30.0;
            }
            Interaction::None => {
                text.sections[0].style.font_size = 25.0;
            }
        }
    }
}

fn volume(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<BGMusic>>,
) {
    if let Ok(sink) = music_controller.get_single() {
        if keyboard_input.just_pressed(KeyCode::Equal) {
            sink.set_volume(sink.volume() + 0.1);
        } else if keyboard_input.just_pressed(KeyCode::Minus) {
            sink.set_volume(sink.volume() - 0.1);
        } else if keyboard_input.just_pressed(KeyCode::Digit0) {
            sink.set_volume(0.0);
        }
    }
}

#[derive(Component)]
struct BGMusic;

fn setup_background_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("music/Lost in the Dessert.ogg"),
            settings: PlaybackSettings {
                volume: Volume::new(0.25),
                mode: PlaybackMode::Loop,
                ..default()
            },
        },
        BGMusic,
    ));
}

fn pause(
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::InGame => {
                next_state.set(GameState::Pause);
            }
            GameState::Pause => {
                next_state.set(GameState::InGame);
            }
            _ => {}
        }
    }
}

fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let title_font: Handle<Font> = asset_server.load("fonts/Kaph-Regular.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                top: Val::Px(-100.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Paused".to_string(),
                    TextStyle {
                        font_size: 60.0,
                        color: Color::WHITE,
                        font: title_font,
                    },
                )
                .with_text_justify(JustifyText::Center),
                UI_LAYER,
                TitleText,
            ));
        });

    let font = asset_server.load("fonts/PeaberryMono.ttf");

    let text_style = TextStyle {
        color: Color::WHITE,
        font_size: 25.0,
        font,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(20.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                ..default()
            },
            UI_LAYER,
        ))
        .with_children(|parent| {
            parent
                .spawn((ButtonBundle {
                    background_color: Color::PURPLE.into(),
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(150.),
                        height: Val::Px(50.),
                        margin: UiRect {
                            top: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back".to_string(),
                        text_style.clone(),
                    ));
                });

            parent
                .spawn((ButtonBundle {
                    background_color: Color::PURPLE.into(),
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(150.),
                        height: Val::Px(50.),
                        margin: UiRect {
                            top: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Give Up".to_string(),
                        text_style.clone(),
                    ));
                });

            parent.spawn((
                TextBundle::from_section(
                    "-/= to lower/raise volume\n0 to mute".to_string(),
                    text_style.clone(),
                )
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    margin: UiRect {
                        top: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                }),
                UI_LAYER,
            ));
        });
}

fn pause_menu_system(
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if text.sections[0].value == "Back" {
                    state.set(GameState::InGame);
                } else if text.sections[0].value == "Give Up" {
                    state.set(GameState::GiveUp);
                }
            }
            Interaction::Hovered => {
                text.sections[0].style.font_size = 30.0;
            }
            Interaction::None => {
                text.sections[0].style.font_size = 25.0;
            }
        }
    }
}

fn cleanup_pause_menu(
    mut commands: Commands,
    interaction_query: Query<(Entity, &Interaction, &mut UiImage), With<Button>>,
    text_query: Query<Entity, With<Text>>,
) {
    for entity in &text_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &mut interaction_query.iter() {
        commands.entity(entity.0).despawn_recursive();
    }
}

fn log_transitions(mut transitions: EventReader<StateTransitionEvent<GameState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.before, transition.after
        );
    }
}

fn setup_give_up_menu(mut commands: Commands, asset_server: Res<AssetServer>, distance_traveled: Res<DistanceTraveled>) {
    let distance = distance_traveled.0 / 64.;
    let title_font: Handle<Font> = asset_server.load("fonts/Kaph-Regular.ttf");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                top: Val::Px(-100.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "You gave up!".to_string(),
                    TextStyle {
                        font_size: 60.0,
                        color: Color::WHITE,
                        font: title_font,
                    },
                )
                .with_text_justify(JustifyText::Center),
                UI_LAYER,
                TitleText,
            ));
        });

    let font = asset_server.load("fonts/PeaberryMono.ttf");

    let text_style = TextStyle {
        color: Color::WHITE,
        font_size: 25.0,
        font,
    };

    if distance > 100. {

        let phrases = [
            "You almost made it!",
            "Nearly there!",
            "So close!",
            "Just a bit more!",
            "You'll get it next time!",
            "You were almost there!",
            "Don't give up so easily!",
            "You were so close!",
            "You were almost there!",
            "Maybe next time!"
        ];

        commands.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(20.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                ..default()
            },
            UI_LAYER,
        )).with_children(|parent| {
            parent.spawn((
            TextBundle::from_section(
                phrases.choose(&mut rand::thread_rng()).unwrap().to_string(),
                text_style.clone(),
            )
            .with_text_justify(JustifyText::Center),
            UI_LAYER,
        ));
    });
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        left: Val::Px(0.),
                        right: Val::Px(0.),
                        top: Val::Px(20.),
                        bottom: Val::Px(0.),
                    },
                    ..default()
                },
                ..default()
            },
            UI_LAYER,
        ))
        .with_children(|parent| {
            parent
                .spawn((ButtonBundle {
                    background_color: Color::PURPLE.into(),
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(200.),
                        height: Val::Px(50.),
                        margin: UiRect {
                            top: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Try again".to_string(),
                        text_style.clone(),
                    ));
                });

            parent
                .spawn((ButtonBundle {
                    background_color: Color::PURPLE.into(),
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(150.),
                        height: Val::Px(50.),
                        margin: UiRect {
                            top: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Quit".to_string(),
                        text_style.clone(),
                    ));
                });
        });
}

fn give_up_menu_system(
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                if text.sections[0].value == "Try again" {
                    state.set(GameState::Cleanup);
                } else if text.sections[0].value == "Quit" {
                    std::process::exit(0);
                }
            }
            Interaction::Hovered => {
                text.sections[0].style.font_size = 30.0;
            }
            Interaction::None => {
                text.sections[0].style.font_size = 25.0;
            }
        }
    }
}

fn cleanup_give_up_menu(
    mut commands: Commands,
    interaction_query: Query<(Entity, &Interaction, &mut UiImage), With<Button>>,
    text_query: Query<Entity, With<Text>>,
) {
    for entity in &text_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &mut interaction_query.iter() {
        commands.entity(entity.0).despawn_recursive();
    }
}

fn cleanup(mut next_state: ResMut<NextState<GameState>>, mut distance_traveled: ResMut<DistanceTraveled>) {
    distance_traveled.0 = 0.;
    next_state.set(GameState::InGame);
}

fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(all(not(feature = "atlas"), feature = "render"))] array_texture_loader: Res<
        ArrayTextureLoader,
    >,
) {
    let texture_handle: Handle<Image> = asset_server.load("textures/starfield.png");
    let map_size = TilemapSize { x: 1024, y: 1024 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 1024., y: 1024. };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..default()
    });

    #[cfg(all(not(feature = "atlas"), feature = "render"))]
    {
        array_texture_loader.add(TilemapArrayTexture {
            texture: TilemapTexture::Single(asset_server.load("textures/Space Background.png")),
            tile_size,
            ..default()
        });
    }
}
