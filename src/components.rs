use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomId(pub usize);

#[derive(Component)]
pub struct MinimapTile;

#[derive(Component)]
pub struct CameraFollow;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Wall;

#[derive(Debug, Clone, Copy, Component)]
pub enum PlayerClass {
    Warrior,
    Mage,
    Ranger,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub i32);