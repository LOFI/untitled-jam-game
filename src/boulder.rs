use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

pub struct BoulderPlugin;

use crate::GameState;

#[derive(Component)]
pub struct Boulder;

impl Plugin for BoulderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::MainMenu), spawn_boulder)
            .add_systems(OnExit(GameState::InGame), freeze_boulder)
            .add_systems(OnEnter(GameState::InGame), unfreeze_boulder);
    }
}

fn freeze_boulder(mut commands: Commands, boulder: Query<Entity, With<Boulder>>) {
    if boulder.is_empty() {
        return;
    }

    commands.entity(boulder.single()).insert(RigidBody::Fixed);
}

fn unfreeze_boulder(mut commands: Commands, boulder: Query<Entity, With<Boulder>>) {
    if boulder.is_empty() {
        return;
    }

    commands.entity(boulder.single()).insert(RigidBody::Dynamic);
}

fn spawn_boulder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(Circle { radius: 64. }).into(),
            material: materials.add(asset_server.load("textures/stone.png")),
            // material: materials.add(Color::BLUE),
            transform: Transform::from_xyz(0.0, 0.0, 5.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(64.))
        .insert(AdditionalMassProperties::Mass(1134.)) // 2500 lbs in kg
        .insert(Boulder);
}
