use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::{plugin::systems::RigidBodyWritebackComponents, prelude::*};

pub struct BoulderPlugin;

use crate::GameState;

#[derive(Component)]
pub struct Boulder;

impl Plugin for BoulderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::MainMenu), spawn_boulder)
            .add_systems(OnEnter(GameState::Pause), freeze_boulder)
            .add_systems(Update, fall);
    }
}

fn freeze_boulder(mut boulder: Query<&mut RigidBodyWritebackComponents, With<Boulder>>) {
    if boulder.is_empty() {
        return;
    }

    let mut boulder = boulder.single_mut();
}

fn spawn_boulder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(Circle { radius: 64. }).into(),
            material: materials.add(Color::GRAY),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(64.))
        .insert(AdditionalMassProperties::Mass(100.0))
        .insert(Boulder);
}

fn fall() {}
