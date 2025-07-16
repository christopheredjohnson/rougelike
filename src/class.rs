use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub enum PlayerClass {
    Paladin,
    #[default]
    Archer,

    Mage,
}

#[derive(Component)]
pub struct ClassStats {
    pub class: PlayerClass,
    pub health: f32,
    pub move_speed: f32,
    pub attack_cooldown: f32,
}
