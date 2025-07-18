use bevy::prelude::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum PlayerClass {
    Warrior,
    Archer,
    Mage,
}

#[derive(Resource)]
pub struct SelectedClass(pub Option<PlayerClass>);