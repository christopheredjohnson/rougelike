use std::collections::HashSet;

use bevy::{color::palettes::css::BLACK, prelude::*};
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            PanCamPlugin::default(),
        ))
        .init_state::<AppState>() // Alternatively we could use .insert_state(AppState::Menu)
        .insert_resource(ClearColor(BLACK.into()))
        .insert_resource(SelectedClass(None))
        .add_systems(Startup, setup)
        // This system runs when we enter `AppState::Menu`, during the `StateTransition` schedule.
        // All systems from the exit schedule of the state we're leaving are run first,
        // and then all systems from the enter schedule of the state we're entering are run second.
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        // By contrast, update systems are stored in the `Update` schedule. They simply
        // check the value of the `State<T>` resource to see if they should run each frame.
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(Update, player_movement.run_if(in_state(AppState::InGame)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(Resource)]
struct MenuData {
    root_entity: Entity,
}

#[derive(Debug, Clone, Copy, Component)]
enum PlayerClass {
    Warrior,
    Mage,
    Ranger,
}

#[derive(Resource)]
struct SelectedClass(pub Option<PlayerClass>);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Wall;


#[derive(Debug, Clone)]
struct Room {
    id: usize,
    bounds: Rect,       // Original BSP split area
    inner: Rect,        // Carved room within bounds
}


const MAP_WIDTH: usize = 64;
const MAP_HEIGHT: usize = 64;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

const FLOOR_TILE_INDEX: usize = 119;
const WALL_VERTICAL_INDEX: usize = 17; // e.g. │ sprite
const WALL_HORIZONTAL_INDEX: usize = 18; // e.g. ─ sprite

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), PanCam::default()));
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    let root_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for (label, class) in [
                ("Warrior", PlayerClass::Warrior),
                ("Mage", PlayerClass::Mage),
                ("Ranger", PlayerClass::Ranger),
            ] {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(200.),
                                height: Val::Px(65.),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        class,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font: font.clone(),
                                font_size: 30.,
                                color: Color::WHITE,
                            },
                        ));
                    });
            }
        })
        .id();

    commands.insert_resource(MenuData { root_entity });
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut selected_class: ResMut<SelectedClass>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PlayerClass),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, class) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                selected_class.0 = Some(*class);
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.root_entity).despawn_recursive();
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
                spawn_floor_tile(&mut commands, x, y, tile_texture.clone(), tile_texture_atlas_layout.clone());
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
    mut player_query: Query<(&mut Transform, &mut Position), (Without<Wall>, With<Player>)>,
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

    // First borrow player and clone position
    let (player_transform, player_pos) = match player_query.get_single() {
        Ok((t, p)) => (t, p),
        Err(_) => return,
    };
    let new_pos = Position {
        x: player_pos.x + delta.0,
        y: player_pos.y + delta.1,
    };

    // Then do wall query AFTER dropping player query borrow
    let blocked = wall_query.iter().any(|&pos| pos == new_pos);

    if blocked {
        return;
    }

    // Now re-borrow player mutably
    if let Ok((mut transform, mut pos)) = player_query.get_single_mut() {
        pos.x = new_pos.x;
        pos.y = new_pos.y;
        transform.translation = Vec3::new(new_pos.x as f32 * 32.0, new_pos.y as f32 * 32.0, 1.0);
    }
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    fn subdivide(&self, rng: &mut impl Rng) -> Option<(Rect, Rect)> {
        let min_size = 6;

        let can_split_h = self.height > min_size * 2;
        let can_split_v = self.width > min_size * 2;

        if !can_split_h && !can_split_v {
            return None;
        }

        let split_horizontal = if can_split_h && can_split_v {
            rng.gen_bool(0.5)
        } else {
            can_split_h
        };

        if split_horizontal {
            let max_split = self.height - min_size;
            let min_split = min_size;
            if min_split < max_split {
                let split = rng.gen_range(min_split..max_split);
                Some((
                    Rect {
                        x: self.x,
                        y: self.y,
                        width: self.width,
                        height: split,
                    },
                    Rect {
                        x: self.x,
                        y: self.y + split,
                        width: self.width,
                        height: self.height - split,
                    },
                ))
            } else {
                None
            }
        } else {
            let max_split = self.width - min_size;
            let min_split = min_size;
            if min_split < max_split {
                let split = rng.gen_range(min_split..max_split);
                Some((
                    Rect {
                        x: self.x,
                        y: self.y,
                        width: split,
                        height: self.height,
                    },
                    Rect {
                        x: self.x + split,
                        y: self.y,
                        width: self.width - split,
                        height: self.height,
                    },
                ))
            } else {
                None
            }
        }
    }
}

fn bsp_split(rect: Rect, depth: u32, rng: &mut impl Rng) -> Vec<Room> {
    let mut leaves = vec![rect];
    for _ in 0..depth {
        let mut next = Vec::new();
        for r in &leaves {
            if let Some((a, b)) = r.subdivide(rng) {
                next.push(a);
                next.push(b);
            } else {
                next.push(*r);
            }
        }
        leaves = next;
    }

    // Carve inner rooms
    leaves
        .into_iter()
        .enumerate()
        .map(|(i, bounds)| {
            let margin_x = rng.gen_range(1..=2);
            let margin_y = rng.gen_range(1..=2);
            let max_width = (bounds.width - 2 * margin_x).max(3);
            let max_height = (bounds.height - 2 * margin_y).max(3);

            let inner = Rect {
                x: bounds.x + margin_x,
                y: bounds.y + margin_y,
                width: max_width,
                height: max_height,
            };

            Room { id: i, bounds, inner }
        })
        .collect()
}

fn spawn_floor_tile(
    commands: &mut Commands,
    x: i32,
    y: i32,
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
) {
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_translation(Vec3::new(
                x as f32 * 32.0,
                y as f32 * 32.0,
                0.0,
            )),
            ..default()
        },
        TextureAtlas {
            layout,
            index: FLOOR_TILE_INDEX,
        },
        Position { x, y },
    ));
}

fn spawn_wall_tile(
    commands: &mut Commands,
    x: i32,
    y: i32,
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
) {
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_translation(Vec3::new(
                x as f32 * 32.0,
                y as f32 * 32.0,
                0.0,
            )),
            ..default()
        },
        TextureAtlas {
            layout,
            index: WALL_HORIZONTAL_INDEX, // You can later change this based on orientation
        },
        Position { x, y },
        Wall,
    ));
}
