use std::collections::HashSet;

use bevy::{color::palettes::css, prelude::*, render::view::RenderLayers};
use rand::Rng;



use crate::{components::*, map::{bsp_split, Rect, Room}, spawn_floor_tile, spawn_wall_tile, AppState, PlayerClass, SelectedClass, FLOOR_TILE_INDEX, MAP_HEIGHT, MAP_WIDTH, MINIMAP_LAYER, WALL_HORIZONTAL_INDEX}; 

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_game)
           .add_systems(
               Update,
               (player_movement, update_minimap_highlight, camera_follow_system)
                   .chain()
                   .run_if(in_state(AppState::InGame))
           );
    }
}



fn setup_game(
    mut commands: Commands,
    selected_class: Res<SelectedClass>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let tile_texture = asset_server.load("tiles.png");
    let tile_layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 17, 26, None, None);
    let tile_texture_atlas_layout = texture_atlas_layouts.add(tile_layout);

    let mut rng = rand::thread_rng();
    let rooms: Vec<Room> = bsp_split(
        Rect {
            x: 0,
            y: 0,
            width: MAP_WIDTH as i32,
            height: MAP_HEIGHT as i32,
        },
        5,
        &mut rng,
    );

    let mut floor_positions = HashSet::new();

    // Spawn rooms
    for room in &rooms {
        for y in room.inner.y..room.inner.y + room.inner.height {
            for x in room.inner.x..room.inner.x + room.inner.width {
                floor_positions.insert(Position { x, y });
                commands.spawn((
                    SpriteBundle {
                        texture: tile_texture.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 * 32.0,
                            y as f32 * 32.0,
                            0.0,
                        )),
                        ..default()
                    },
                    TextureAtlas {
                        layout: tile_texture_atlas_layout.clone(),
                        index: FLOOR_TILE_INDEX,
                    },
                    Position { x, y },
                    RoomId(room.id),
                ));
            }
        }
    }

    // Spawn corridors
    for i in 1..rooms.len() {
        let (x1, y1) = rooms[i - 1].inner.center();
        let (x2, y2) = rooms[i].inner.center();

        if rng.gen_bool(0.5) {
            for x in x1.min(x2)..=x1.max(x2) {
                floor_positions.insert(Position { x, y: y1 });
                spawn_floor_tile(
                    &mut commands,
                    x,
                    y1,
                    tile_texture.clone(),
                    tile_texture_atlas_layout.clone(),
                );
            }
            for y in y1.min(y2)..=y1.max(y2) {
                floor_positions.insert(Position { x: x2, y });
                spawn_floor_tile(
                    &mut commands,
                    x2,
                    y,
                    tile_texture.clone(),
                    tile_texture_atlas_layout.clone(),
                );
            }
        } else {
            for y in y1.min(y2)..=y1.max(y2) {
                floor_positions.insert(Position { x: x1, y });
                spawn_floor_tile(
                    &mut commands,
                    x1,
                    y,
                    tile_texture.clone(),
                    tile_texture_atlas_layout.clone(),
                );
            }
            for x in x1.min(x2)..=x1.max(x2) {
                floor_positions.insert(Position { x, y: y2 });
                spawn_floor_tile(
                    &mut commands,
                    x,
                    y2,
                    tile_texture.clone(),
                    tile_texture_atlas_layout.clone(),
                );
            }
        }
    }

    // Spawn walls around floors
    for pos in &floor_positions {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let neighbor = Position {
                    x: pos.x + dx,
                    y: pos.y + dy,
                };
                if !floor_positions.contains(&neighbor) {
                    spawn_wall_tile(
                        &mut commands,
                        neighbor.x,
                        neighbor.y,
                        tile_texture.clone(),
                        tile_texture_atlas_layout.clone(),
                    );
                }
            }
        }
    }

    let minimap_tile_size = 4.0;
    let minimap_offset = Vec2::new(
        -(MAP_WIDTH as f32 * minimap_tile_size) / 2.0,
        -(MAP_HEIGHT as f32 * minimap_tile_size) / 2.0,
    );

    for room in &rooms {
        for y in room.inner.y..room.inner.y + room.inner.height {
            for x in room.inner.x..room.inner.x + room.inner.width {
                let minimap_pos = Vec3::new(
                    x as f32 * minimap_tile_size + minimap_offset.x,
                    y as f32 * minimap_tile_size + minimap_offset.y,
                    10.0, // Render on top
                );

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: css::DARK_GRAY.into(),
                            custom_size: Some(Vec2::splat(minimap_tile_size)),
                            ..default()
                        },
                        transform: Transform::from_translation(minimap_pos),
                        ..default()
                    },
                    RoomId(room.id),
                    MinimapTile,
                    Position { x, y },
                    RenderLayers::layer(MINIMAP_LAYER),
                ));
            }
        }
    }
    // Spawn player in center of first room
    if let Some(class) = selected_class.0 {
        let texture = asset_server.load("rogues.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 7, 7, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        let index = match class {
            PlayerClass::Mage => 29,
            PlayerClass::Warrior => 0,
            PlayerClass::Ranger => 2,
        };

        let (x, y) = rooms[0].inner.center();
        commands.spawn((
            SpriteBundle {
                texture: texture.clone(),
                transform: Transform::from_translation(Vec3::new(
                    x as f32 * 32.0,
                    y as f32 * 32.0,
                    1.0,
                )),
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout,
                index,
            },
            Position { x, y },
            Player,
        ));
    } else {
        panic!("No class selected!");
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Position), (Without<Wall>, With<Player>)>,
        Query<&mut Transform, (With<MinimapTile>, With<Player>)>,
    )>,
    wall_query: Query<&Position, With<Wall>>,
) {
    let mut delta = (0, 0);
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        delta.1 += 1;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        delta.1 -= 1;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        delta.0 -= 1;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        delta.0 += 1;
    }

    if delta == (0, 0) {
        return;
    }

    let mut player_query = param_set.p0();

    let (player_transform, player_pos) = match player_query.get_single() {
        Ok((t, p)) => (t, p),
        Err(_) => return,
    };

    let new_pos = Position {
        x: player_pos.x + delta.0,
        y: player_pos.y + delta.1,
    };

    let blocked = wall_query.iter().any(|&pos| pos == new_pos);
    if blocked {
        return;
    }

    let minimap_tile_size = 4.0;
    let minimap_offset = Vec2::new(
        -(MAP_WIDTH as f32 * minimap_tile_size) / 2.0,
        -(MAP_HEIGHT as f32 * minimap_tile_size) / 2.0,
    );

    if let Ok((mut transform, mut pos)) = player_query.get_single_mut() {
        pos.x = new_pos.x;
        pos.y = new_pos.y;
        transform.translation = Vec3::new(new_pos.x as f32 * 32.0, new_pos.y as f32 * 32.0, 1.0);
    }

    let mut minimap_query = param_set.p1();
    if let Ok(mut mini_transform) = minimap_query.get_single_mut() {
        mini_transform.translation = Vec3::new(
            new_pos.x as f32 * minimap_tile_size + minimap_offset.x,
            new_pos.y as f32 * minimap_tile_size + minimap_offset.y,
            11.0,
        );
    }
}


fn update_minimap_highlight(
    player_query: Query<&Position, With<Player>>,
    floor_query: Query<(&Position, &RoomId)>,
    mut minimap_tiles: Query<(&RoomId, &mut Sprite), With<MinimapTile>>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };

    // Find the player's current room
    let mut current_room_id = None;
    for (pos, room_id) in floor_query.iter() {
        if pos == player_pos {
            current_room_id = Some(*room_id);
            break;
        }
    }

    // Highlight tiles in the same room, reset others
    for (room_id, mut sprite) in &mut minimap_tiles {
        if Some(*room_id) == current_room_id {
            sprite.color = css::YELLOW.into(); // highlighted color
        } else {
            sprite.color = css::DARK_GRAY.into(); // default color
        }
    }
}

fn camera_follow_system(
    player_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<CameraFollow>, Without<Player>)>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    camera_transform.translation.x = player_pos.x as f32 * 32.0;
    camera_transform.translation.y = player_pos.y as f32 * 32.0;
}