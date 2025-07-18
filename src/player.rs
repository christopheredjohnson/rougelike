use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum PlayerClass {
    Warrior,
    Archer,
    Mage,
}

#[derive(Resource)]
pub struct SelectedClass(pub Option<PlayerClass>);

#[derive(Component)]
pub struct Player;

pub fn rotate_player_to_mouse(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    let (camera, camera_transform) = camera_q.single();
    let window = windows.single();

    if let Some(screen_pos) = window.cursor_position() {
        // Convert screen space to world space
        if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
            let mut player_transform = player_q.single_mut();
            let player_pos = player_transform.translation.truncate();

            // Calculate direction and angle
            let direction = world_pos - player_pos;
            let angle = direction.y.atan2(direction.x); // angle in radians

            // Apply rotation (around Z axis in 2D)
            player_transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

pub fn player_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let mut velocity = query.single_mut();
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    let speed = 200.0;
    velocity.linvel = direction.normalize_or_zero() * speed;
}
