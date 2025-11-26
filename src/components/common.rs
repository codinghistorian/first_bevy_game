use bevy::prelude::*;

/// A component to store an entity's health.
#[derive(Component)]
pub struct Hp {
    pub current: f32,
    pub max: f32,
}

/// A marker component for the health bar's fill, linking it to the entity it represents.
#[derive(Component)]
pub struct HealthBar {
    pub entity: Entity,
}

