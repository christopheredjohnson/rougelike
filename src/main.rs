use std::collections::HashSet;

use bevy::render::view::RenderLayers;
use bevy::{
    color::palettes::css::{self, BLACK},
    prelude::*,
};
use rand::Rng;

use crate::components::*;
use crate::game::{GamePlugin};
use crate::menu::MenuPlugin;

mod menu;
mod map;
mod game;
mod components;

pub const MINIMAP_LAYER: usize = 1;
pub const MAP_WIDTH: usize = 64;
pub const MAP_HEIGHT: usize = 64;

pub const FLOOR_TILE_INDEX: usize = 119;
pub const WALL_VERTICAL_INDEX: usize = 17; // e.g. │ sprite
pub const  WALL_HORIZONTAL_INDEX: usize = 18; // e.g. ─ sprite

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(Debug, Clone, Copy, Component)]
pub enum PlayerClass {
    Warrior,
    Mage,
    Ranger,
}

#[derive(Resource)]
pub struct SelectedClass(pub Option<PlayerClass>);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MenuPlugin,
            GamePlugin
        ))
        .insert_resource(ClearColor(BLACK.into()))
        .insert_resource(SelectedClass(None))
        .add_systems(Startup, setup)
        .run();
}



fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraFollow,
        RenderLayers::layer(0), // Main layer only
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1, // draw after main world camera
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)),
            ..default()
        },
        RenderLayers::layer(MINIMAP_LAYER),
    ));
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
