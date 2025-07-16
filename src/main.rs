use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

use crate::class::{ClassStats, PlayerClass};

mod class;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Hammerwatch-like".into(),
                        resolution: (800., 600.).into(),
                        cursor: bevy::window::Cursor {
                            visible: false, // Hide the default cursor
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((
            WorldInspectorPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup, load_projectile_assets, setup_crosshair))
        .add_systems(
            Update,
            (
                player_movement,
                player_attack,
                enemy_ai_system,
                enemy_attack_system,
                projectile_despawn,
                hit_enemy,
                hit_player,
                melee_despawn,
                camera_follow,
                animate_projectiles,
                rotate_toward_mouse,
                update_crosshair,
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
struct EnemyProjectile;

#[derive(Component)]
struct MeleeAttack;

#[derive(Component)]
struct EnemyMeleeAttack;

#[derive(Component)]
struct MeleeLifetime(Timer);

#[derive(Component)]
struct AttackTimer {
    timer: Timer,
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct EnemyAI {
    detection_range: f32,
    attack_range: f32,
    move_speed: f32,
    attack_cooldown: f32,
    last_attack: f32,
}

impl Default for EnemyAI {
    fn default() -> Self {
        Self {
            detection_range: 200.0,
            attack_range: 60.0,
            move_speed: 75.0,
            attack_cooldown: 2.0,
            last_attack: 0.0,
        }
    }
}

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct Crosshair;

#[derive(Resource)]
struct ProjectileAssets {
    fireball_texture: Handle<Image>,
    fireball_layout: Handle<TextureAtlasLayout>,
    arrow_texture: Handle<Image>,
}

#[derive(Component)]
struct AnimatedProjectile {
    timer: Timer,
}

fn load_projectile_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let fireball_texture: Handle<Image> =
        asset_server.load("Magic(Projectile)/Wizard-Attack02_Effect.png");
    let arrow_texture: Handle<Image> = asset_server.load("Arrow(Projectile)/Arrow01(100x100).png");

    // 7 columns, 1 row, each 100x100 pixels
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 7, 1, None, None);
    let fireball_layout = layouts.add(layout);

    commands.insert_resource(ProjectileAssets {
        fireball_texture: fireball_texture,
        fireball_layout: fireball_layout,
        arrow_texture: arrow_texture,
    });
}

fn setup_crosshair(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create crosshair made of two rectangles (horizontal and vertical lines)
    let crosshair_material = materials.add(ColorMaterial::from(Color::srgba(1.0, 1.0, 1.0, 0.8)));

    // Horizontal line
    let horizontal_mesh = meshes.add(Mesh::from(Rectangle::new(20.0, 2.0)));
    // Vertical line
    let vertical_mesh = meshes.add(Mesh::from(Rectangle::new(2.0, 20.0)));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(horizontal_mesh),
            material: crosshair_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 10.0), // High z-index to render on top
            ..default()
        },
        Crosshair,
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(vertical_mesh),
            material: crosshair_material,
            transform: Transform::from_xyz(0.0, 0.0, 10.0), // High z-index to render on top
            ..default()
        },
        Crosshair,
    ));
}

fn update_crosshair(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut crosshair_q: Query<&mut Transform, With<Crosshair>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let world_pos = ray.origin.truncate();

    // Update all crosshair components to follow the mouse
    for mut transform in &mut crosshair_q {
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
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
        Health {
            current: 100.0,
            max: 100.0,
        },
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
            EnemyAI::default(),
            Health {
                current: 3.0,
                max: 3.0,
            },
            RigidBody::Dynamic,
            Collider::cuboid(14.0, 14.0),
            Velocity::zero(),
            GravityScale(0.0),
            CollisionGroups::new(Group::GROUP_3, Group::ALL), // Enemies in group 3
        ));
    }
}

// === AI System ===
fn enemy_ai_system(
    mut enemy_query: Query<(&mut Transform, &mut Velocity, &mut EnemyAI), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    
    let player_pos = player_transform.translation.truncate();
    let current_time = time.elapsed_seconds();

    for (mut enemy_transform, mut enemy_velocity, mut ai) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();
        let distance_to_player = enemy_pos.distance(player_pos);

        // Check if player is within detection range
        if distance_to_player <= ai.detection_range {
            let direction = (player_pos - enemy_pos).normalize();
            
            // If player is within attack range, stop moving and prepare to attack
            if distance_to_player <= ai.attack_range {
                enemy_velocity.linvel = Vec2::ZERO;
                
                // Face the player
                let angle = direction.y.atan2(direction.x);
                enemy_transform.rotation = Quat::from_rotation_z(angle);
            } else {
                // Move towards player
                enemy_velocity.linvel = direction * ai.move_speed;
                
                // Face movement direction
                let angle = direction.y.atan2(direction.x);
                enemy_transform.rotation = Quat::from_rotation_z(angle);
            }
        } else {
            // Player not detected, stop moving
            enemy_velocity.linvel = Vec2::ZERO;
        }
    }
}

fn enemy_attack_system(
    mut commands: Commands,
    mut enemy_query: Query<(&Transform, &mut EnemyAI), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    
    let player_pos = player_transform.translation.truncate();
    let current_time = time.elapsed_seconds();

    for (enemy_transform, mut ai) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();
        let distance_to_player = enemy_pos.distance(player_pos);

        // Check if player is within attack range and cooldown is ready
        if distance_to_player <= ai.attack_range && 
           current_time - ai.last_attack >= ai.attack_cooldown {
            
            let direction = (player_pos - enemy_pos).normalize();
            let spawn_pos = enemy_pos + direction * 20.0;

            // Create enemy projectile (red fireball)
            let projectile_mesh = meshes.add(Mesh::from(Circle::new(8.0)));
            let projectile_material = materials.add(ColorMaterial::from(Color::from(css::RED)));

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(projectile_mesh),
                    material: projectile_material,
                    transform: Transform::from_translation(spawn_pos.extend(1.0)),
                    ..default()
                },
                EnemyProjectile,
                RigidBody::Dynamic,
                Collider::ball(8.0),
                Velocity::linear(direction * 200.0),
                Sleeping::disabled(),
                GravityScale(0.0),
                Sensor,
                CollisionGroups::new(Group::GROUP_4, Group::GROUP_1), // Enemy projectiles hit player
                ActiveEvents::COLLISION_EVENTS,
            ));

            ai.last_attack = current_time;
        }
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
    time: Res<Time>,
    projectile_assets: Res<ProjectileAssets>,
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
        PlayerClass::Mage => {
            let spawn_pos = player_pos + direction * 20.0;

            commands.spawn((
                TextureAtlas {
                    layout: projectile_assets.fireball_layout.clone(),
                    index: 0,
                },
                SpriteBundle {
                    texture: projectile_assets.fireball_texture.clone(),
                    transform: Transform::from_translation(spawn_pos.extend(1.0)),
                    ..default()
                },
                AnimatedProjectile {
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
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
        PlayerClass::Archer => {
            let spawn_pos = player_pos + direction * 20.0;
            let angle = direction.y.atan2(direction.x);

            commands.spawn((
                SpriteBundle {
                    texture: projectile_assets.arrow_texture.clone(),
                    transform: Transform::from_translation(spawn_pos.extend(1.0))
                        .with_rotation(Quat::from_rotation_z(angle)),
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
                CollisionGroups::new(Group::GROUP_2, Group::GROUP_3),
                ActiveEvents::COLLISION_EVENTS,
                MeleeLifetime(Timer::from_seconds(0.1, TimerMode::Once)),
            ));
        }
    }
}

fn projectile_despawn(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<Projectile>, Without<EnemyProjectile>)>,
) {
    for (entity, transform) in &query {
        if transform.translation.length() > 2000.0 {
            commands.entity(entity).despawn();
        }
    }
}

// === Combat Systems ===
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

fn hit_player(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<(Entity, &mut Health), With<Player>>,
    enemy_projectile_query: Query<Entity, With<EnemyProjectile>>,
    enemy_melee_query: Query<Entity, With<EnemyMeleeAttack>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (maybe_player, maybe_hitbox) = match (
                player_query.get(*e1).ok().map(|_| *e1),
                player_query.get(*e2).ok().map(|_| *e2),
            ) {
                (Some(player), _)
                    if enemy_projectile_query.get(*e2).is_ok() || enemy_melee_query.get(*e2).is_ok() =>
                {
                    (player, *e2)
                }
                (_, Some(player))
                    if enemy_projectile_query.get(*e1).is_ok() || enemy_melee_query.get(*e1).is_ok() =>
                {
                    (player, *e1)
                }
                _ => continue,
            };

            // Despawn enemy projectile or melee hitbox
            commands.entity(maybe_hitbox).despawn();

            // Damage player
            if let Ok((_, mut health)) = player_query.get_mut(maybe_player) {
                health.current -= 10.0;
                if health.current <= 0.0 {
                    // Player died - could restart game, show game over screen, etc.
                    println!("Player died!");
                    health.current = 0.0;
                }
            }
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

fn animate_projectiles(
    time: Res<Time>,
    mut query: Query<(&mut TextureAtlas, &mut AnimatedProjectile)>,
) {
    for (mut atlas, mut anim) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            atlas.index = (atlas.index + 1) % 7;
        }
    }
}

fn rotate_toward_mouse(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform, With<Player>>, // or With<Projectile> etc.
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let target = ray.origin.truncate();

    for mut transform in &mut query {
        let position = transform.translation.truncate();
        let direction = target - position;
        let angle = direction.y.atan2(direction.x); // angle in radians

        transform.rotation = Quat::from_rotation_z(angle);
    }
}