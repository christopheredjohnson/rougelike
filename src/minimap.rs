use bevy::{
    color::palettes::css,
    prelude::*,
    render::view::RenderLayers,
};

use crate::{
    components::{MinimapTile, Player, Position, RoomId},
    map::Room,
    AppState, MAP_HEIGHT, MAP_WIDTH, MINIMAP_LAYER,
};

/// Plugin that handles minimap tile rendering and real-time room highlighting.
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_minimap_highlight.run_if(in_state(AppState::InGame)),
        );
    }
}

fn update_minimap_highlight(
    player_query: Query<&Position, With<Player>>,
    floor_query: Query<(&Position, &RoomId)>,
    mut minimap_tiles: Query<(&RoomId, &mut BackgroundColor), With<MinimapTile>>,
) {
    let Ok(player_pos) = player_query.get_single() else { return };

    // Determine current room based on player position
    let mut current_room_id = None;
    for (pos, room_id) in floor_query.iter() {
        if pos == player_pos {
            current_room_id = Some(*room_id);
            break;
        }
    }

    // Highlight UI tiles
    for (room_id, mut bg_color) in &mut minimap_tiles {
        bg_color.0 = if Some(*room_id) == current_room_id {
            css::YELLOW.into()
        } else {
            css::DARK_GRAY.into()
        };
    }
}

pub fn spawn_minimap_ui_tiles(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    rooms: &[Room],
) {
    let tile_size = 4.0;

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                width: Val::Px(MAP_WIDTH as f32 * tile_size),
                height: Val::Px(MAP_HEIGHT as f32 * tile_size),
                flex_wrap: FlexWrap::NoWrap,
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..default()
        })
        .id();

    for room in rooms {
        for y in room.inner.y..room.inner.y + room.inner.height {
            for x in room.inner.x..room.inner.x + room.inner.width {
                commands.entity(container).with_children(|parent| {
                    parent.spawn((
                        ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                left: Val::Px(x as f32 * tile_size),
                                top: Val::Px((MAP_HEIGHT as i32 - 1 - y) as f32 * tile_size),
                                width: Val::Px(tile_size),
                                height: Val::Px(tile_size),
                                ..default()
                            },
                            background_color: css::DARK_GRAY.into(),
                            ..default()
                        },
                        RoomId(room.id),
                        MinimapTile,
                        Position { x, y },
                    ));
                });
            }
        }
    }
}
