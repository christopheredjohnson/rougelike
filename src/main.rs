use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

use crate::class::{ClassStats, PlayerClass};

mod class;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hammerwatch-like".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        
        .add_plugins((
            WorldInspectorPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                player_shoot,
                projectile_movement,
                camera_follow,
            ),
        )
        .run();
}

// === Components ===
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct AttackTimer {
    timer: Timer,
}

// === Setup ===
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let chosen_class = PlayerClass::default();

    let stats = match chosen_class {
        PlayerClass::Paladin => ClassStats {
            class: PlayerClass::Paladin,
            health: 150.0,
            move_speed: 150.0,
            attack_cooldown: 1.0,
        },
        PlayerClass::Archer => ClassStats {
            class: PlayerClass::Archer,
            health: 100.0,
            move_speed: 200.0,
            attack_cooldown: 0.5,
        },
        PlayerClass::Mage => ClassStats {
            class: PlayerClass::Mage,
            health: 80.0,
            move_speed: 180.0,
            attack_cooldown: 0.8,
        },
    };

    // Create a 32x32 green rectangle for the player
    let player_mesh = meshes.add(Mesh::from(Rectangle::new(32.0, 32.0)));
    let player_material = materials.add(ColorMaterial::from(Color::from(css::ORANGE_RED)));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(player_mesh),
            material: player_material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Player,
        AttackTimer {
            timer: Timer::from_seconds(stats.attack_cooldown, TimerMode::Repeating),
        },
        stats,
    ));
}

// === Movement ===
fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &ClassStats), With<Player>>,
    time: Res<Time>,
) {
    let (mut transform, stats) = query.single_mut();
    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        transform.translation += (direction * stats.move_speed * time.delta_seconds()).extend(0.0);
    }
}

fn player_shoot(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut player_q: Query<(&Transform, &ClassStats, &mut AttackTimer), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    let (camera, cam_transform) = camera_q.single();
    let window = windows.single();

    let (player_transform, stats, mut attack_timer) = player_q.single_mut();

    // Tick the timer
    attack_timer.timer.tick(time.delta());

    if buttons.pressed(MouseButton::Left) && attack_timer.timer.finished() {
        if let Some(cursor_pos) = window.cursor_position() {
            let cursor_world = camera
                .viewport_to_world(cam_transform, cursor_pos)
                .unwrap()
                .origin
                .truncate();

            let player_pos = player_transform.translation.truncate();
            let direction = (cursor_world - player_pos).normalize();

            // Create a small projectile (10x10)
            let proj_mesh = meshes.add(Mesh::from(Rectangle::new(10.0, 10.0)));
            let proj_material = materials.add(ColorMaterial::from(Color::from(css::ORANGE_RED)));

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(proj_mesh),
                    material: proj_material,
                    transform: Transform::from_translation(player_pos.extend(1.0)),
                    ..default()
                },
                Projectile,
                Velocity(direction * 400.0),
            ));
        }
    }
}

// === Projectile Movement ===
fn projectile_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Velocity), With<Projectile>>,
    time: Res<Time>,
) {
    for (entity, mut transform, velocity) in &mut query {
        transform.translation += (velocity.0 * time.delta_seconds()).extend(0.0);

        if transform.translation.length() > 2000.0 {
            commands.entity(entity).despawn();
        }
    }
}

// === Camera Follow ===
fn camera_follow(
    player_q: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_q: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_transform = player_q.single();
    let mut camera_transform = camera_q.single_mut();
    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}
