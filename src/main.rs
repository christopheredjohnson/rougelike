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
        .add_systems(Startup, (setup, load_projectile_assets))
        .add_systems(
            Update,
            (
                player_movement,
                player_attack,
                projectile_despawn,
                hit_enemy,
                melee_despawn,
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
struct MeleeAttack;

#[derive(Component)]
struct MeleeLifetime(Timer);

#[derive(Component)]
struct AttackTimer {
    timer: Timer,
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Health {
    current: f32,
}

#[derive(Resource)]
struct ProjectileAssets {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

fn load_projectile_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("Magic(Projectile)/Wizard-Attack02_Effect.png");

    // 7 columns, 1 row, each 100x100 pixels
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 7, 1, None, None);
    let layout_handle = layouts.add(layout);

    commands.insert_resource(ProjectileAssets {
        texture,
        layout: layout_handle,
    });
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
        RigidBody::Dynamic,
        Collider::cuboid(16.0, 16.0),
        Velocity::zero(),
        GravityScale(0.0),
        CollisionGroups::new(Group::GROUP_1, Group::ALL), // Player belongs to group 1
    ));

    for i in 0..5 {
        let x = i as f32 * 100.0 - 200.0;
        let y = 100.0;

        let enemy_mesh = meshes.add(Mesh::from(Rectangle::new(28.0, 28.0)));
        let enemy_material = materials.add(ColorMaterial::from(Color::from(css::DARK_GREEN)));

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(enemy_mesh),
                material: enemy_material,
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            Enemy,
            Health { current: 3.0 },
            RigidBody::Fixed,
            Collider::cuboid(14.0, 14.0),
            CollisionGroups::new(Group::GROUP_3, Group::ALL), // Enemies in group 3
        ));
    }
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

fn player_attack(
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
    attack_timer.timer.tick(time.delta());

    if !buttons.pressed(MouseButton::Left) || !attack_timer.timer.finished() {
        return;
    }

    let player_pos = player_transform.translation.truncate();
    let cursor_world = if let Some(cursor_pos) = window.cursor_position() {
        camera
            .viewport_to_world(cam_transform, cursor_pos)
            .unwrap()
            .origin
            .truncate()
    } else {
        return;
    };

    let direction = (cursor_world - player_pos).normalize();

    match stats.class {
        PlayerClass::Archer | PlayerClass::Mage => {
            // Ranged Projectile
            let proj_mesh = meshes.add(Mesh::from(Rectangle::new(10.0, 10.0)));
            let proj_material = materials.add(ColorMaterial::from(Color::from(css::ORANGE_RED)));
            let spawn_pos = player_pos + direction * 20.0;

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(proj_mesh),
                    material: proj_material,
                    transform: Transform::from_translation(spawn_pos.extend(1.0)),
                    ..default()
                },
                Projectile,
                RigidBody::Dynamic,
                Collider::ball(5.0),
                Velocity::linear(direction * 400.0),
                Sleeping::disabled(),
                GravityScale(0.0),
                CollisionGroups::new(Group::GROUP_2, Group::GROUP_3),
                ActiveEvents::COLLISION_EVENTS,
            ));
        }

        PlayerClass::Paladin => {
            let spawn_pos = player_pos + direction * 20.0;

            commands.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(spawn_pos.extend(1.0)),
                    ..default()
                },
                MeleeAttack,
                RigidBody::Dynamic,
                Collider::cuboid(20.0, 20.0),
                GravityScale(0.0),
                Sensor,
                Velocity::linear(direction * 10.0),
                ActiveEvents::COLLISION_EVENTS,
                MeleeLifetime(Timer::from_seconds(0.1, TimerMode::Once)),
            ));
        }
    }
}

fn projectile_despawn(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Projectile>>,
) {
    for (entity, transform) in &query {
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

fn hit_enemy(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(Entity, &mut Health), With<Enemy>>,
    projectile_query: Query<Entity, With<Projectile>>,
    melee_query: Query<Entity, With<MeleeAttack>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (maybe_enemy, maybe_hitbox) = match (
                enemy_query.get(*e1).ok().map(|_| *e1),
                enemy_query.get(*e2).ok().map(|_| *e2),
            ) {
                (Some(enemy), _)
                    if projectile_query.get(*e2).is_ok() || melee_query.get(*e2).is_ok() =>
                {
                    (enemy, *e2)
                }
                (_, Some(enemy))
                    if projectile_query.get(*e1).is_ok() || melee_query.get(*e1).is_ok() =>
                {
                    (enemy, *e1)
                }
                _ => continue,
            };

            // Despawn projectile or melee hitbox
            commands.entity(maybe_hitbox).despawn();

            // Damage enemy
            if let Ok((_, mut health)) = enemy_query.get_mut(maybe_enemy) {
                health.current -= 1.0;
                if health.current <= 0.0 {
                    commands.entity(maybe_enemy).despawn();
                }
            }
        }
    }
}

fn melee_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MeleeLifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
