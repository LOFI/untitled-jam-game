use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{player::Player, GameState, WINDOW_BOTTOM_Y, WINDOW_HEIGHT, WINDOW_WIDTH};

const COLOR_FLOOR: Color = Color::DARK_GREEN;

pub struct GroundPlugin;

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Foreground;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (spawn_foreground, spawn_ground))
            .add_systems(
                FixedUpdate,
                (expand, keep_centered).run_if(in_state(GameState::InGame)),
            );
    }
}

fn spawn_foreground(mut commands: Commands, ground_query: Query<&Ground>) {
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
                translation: Vec3::new(0., WINDOW_BOTTOM_Y - WINDOW_HEIGHT / 4. - 12., 0.),
                scale: Vec3::new(WINDOW_WIDTH * 2., WINDOW_HEIGHT / 2., 1.),
                rotation: Quat::from_rotation_z(7.5_f32.to_radians()),
            },
            ..default()
        })
        .insert(Foreground);
}

fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(Plane3d {
                    normal: Direction3d::Y,
                })
                .into(),
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
        .insert(Friction::coefficient(0.7))
        .insert(Damping { linear_damping: 0.7, angular_damping: 0.7 })
        .insert(Collider::cuboid(WINDOW_WIDTH, 1.));
}

fn expand(
    mut query: Query<&mut Transform, With<Ground>>,
    player_query: Query<&KinematicCharacterControllerOutput, With<crate::player::Player>>,
) {
    if player_query.is_empty() || query.is_empty() {
        return;
    }
    let player = player_query.single();

    for mut ground in &mut query {
        ground.scale.x += player.effective_translation.x.abs();
    }
}

fn keep_centered(
    mut query: Query<(&mut Transform, &Foreground), Without<Player>>,
    player_query: Query<
        (&Transform, &KinematicCharacterControllerOutput),
        With<crate::player::Player>,
    >,
) {
    if player_query.is_empty() || query.is_empty() {
        return;
    }
    let (player, output) = player_query.single();
    let (mut ground, _) = query.single_mut();

    ground.translation.x = player.translation.x;
    if output.grounded {
        ground.translation.y = player.translation.y - WINDOW_HEIGHT / 4. - 24.;
    }
}
