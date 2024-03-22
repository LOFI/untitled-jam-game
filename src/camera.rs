use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_parallax::{
    CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, ParallaxCameraComponent,
    RepeatStrategy,
};
use bevy_rapier2d::prelude::*;

use crate::{player::Player, GameState, WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, move_camera);
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct UICamera;

pub const UI_LAYER: RenderLayers = RenderLayers::layer(9);

fn spawn_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
) {
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

    let camera = commands
        .spawn((Camera2dBundle::default(), MainCamera))
        .insert(ParallaxCameraComponent { render_layer: 0 })
        .id();

    create_parallax.send(CreateParallaxEvent {
        camera,
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0., 0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorHorizontally),
                path: "layers/sky.png".to_string(),
                tile_size: Vec2::new(384., 216.),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(5.),
                z: 0.,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.9, 0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorHorizontally),
                path: "layers/far_mountains.png".to_string(),
                tile_size: Vec2::new(384., 216.),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(5.),
                z: 0.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.7, 0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorHorizontally),
                path: "layers/grassy_mountains.png".to_string(),
                tile_size: Vec2::new(384., 216.),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(5.),
                z: 1.,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.5, 0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorHorizontally),
                path: "layers/clouds_mid.png".to_string(),
                tile_size: Vec2::new(384., 216.),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(5.),
                z: 1.5,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.3, 0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorHorizontally),
                path: "layers/hill.png".to_string(),
                tile_size: Vec2::new(384., 216.),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(5.),
                z: 2.,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.1, 0.),
                repeat: LayerRepeat::horizontally(RepeatStrategy::MirrorHorizontally),
                path: "layers/clouds_front.png".to_string(),
                tile_size: Vec2::new(384., 216.),
                cols: 1,
                rows: 1,
                scale: Vec2::splat(5.),
                z: 2.5,
                ..default()
            },
        ],
    });

    next_state.set(GameState::MainMenu);
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
    camera.0.translation.y = transform.translation.y + WINDOW_HEIGHT / 5.;
}
