use bevy::{prelude::*, render::mesh::shape::Plane, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{GameState, WINDOW_BOTTOM_Y, WINDOW_HEIGHT, WINDOW_WIDTH};

const COLOR_FLOOR: Color = Color::GREEN;

pub struct GroundPlugin;

#[derive(Component)]
struct Ground;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), spawn_ground)
            .add_systems(FixedUpdate, expand.run_if(in_state(GameState::InGame)));
    }
}

fn spawn_floor(mut commands: Commands, asset_server: Res<AssetServer>, ground_query: Query<&Ground>) {
    if !ground_query.is_empty() {
        return;
    }

    // Slope
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_FLOOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., WINDOW_BOTTOM_Y, 0.),
                scale: Vec3::new(WINDOW_WIDTH * 2., WINDOW_HEIGHT / 2., 1.),
                rotation: Quat::from_rotation_z(7.5_f32.to_radians()),
            },
            ..default()
        })
        .insert(Ground)
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(0.5, 0.5));
}

fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(Plane3d { normal: Direction3d::Y }).into(),
            material: materials.add(Color::DARK_GREEN),
            transform: Transform {
                translation: Vec3::new(0., WINDOW_BOTTOM_Y, 0.),
                rotation: Quat::from_rotation_z(7.5_f32.to_radians()),
                ..default()
            },
            ..default()
        })
        .insert(Ground)
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(WINDOW_WIDTH, 5.));
}

fn expand(
    mut query: Query<&mut Transform, With<Ground>>,
    player_query: Query<&KinematicCharacterControllerOutput, With<crate::player::Player>>,
) {
    if player_query.is_empty() || query.is_empty() {
        return;
    }
    let player = player_query.single();
    let mut ground = query.single_mut();

    if player.desired_translation.x != 0. {
        ground.scale.x += 1.;
    }

}
