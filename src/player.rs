use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (fall, movement));
    }
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                anchor: bevy::sprite::Anchor::BottomCenter,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(KinematicCharacterController::default())
        .insert(Collider::cuboid(0.5, 0.5));
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

    if input.just_pressed(KeyCode::KeyD) || input.just_pressed(KeyCode::ArrowRight) {
        movement += time.delta_seconds() * 100.0;
    }
    if input.just_pressed(KeyCode::KeyA) || input.just_pressed(KeyCode::ArrowLeft) {
        movement -= time.delta_seconds() * 100.0;
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.0)),
    }
}
