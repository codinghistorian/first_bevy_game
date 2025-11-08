use bevy::prelude::*;

/// Marker component for the player character
#[derive(Component)]
pub struct Player;

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

/// Component to track player velocity (for jumping and gravity)
#[derive(Component)]
pub struct PlayerVelocity {
    pub y: f32,
    pub jump_type: JumpType,
    pub facing_direction: Vec2,
}

/// Component to track jump charging (hold duration)
#[derive(Component)]
pub struct JumpCharge {
    pub timer: f32,
    pub is_charging: bool,
}

/// Type of jump the player is currently performing
#[derive(Clone, Copy, PartialEq)]
pub enum JumpType {
    None,
    High,
    Small,
}

/// Marker component for the floor/platform
#[derive(Component)]
pub struct Floor;

/// Component to track dashing state
#[derive(Component)]
pub struct Dash {
    pub timer: f32,
    pub direction: f32,
}

/// Component for projectiles
#[derive(Component)]
pub struct Projectile {
    pub direction: Vec2,
}

/// Component to track shooting cooldown
#[derive(Component)]
pub struct Shooting {
    pub timer: f32,
}

/// Component to track invincibility frames (prevents damage spam)
#[derive(Component)]
pub struct Invincibility {
    pub timer: f32,
}