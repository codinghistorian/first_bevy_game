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

/// A mask component for the circular player HP bar. This is used to hide the top
/// portion of the circular fill so the bar appears to drain from the top in a
/// linear, damage-proportional way.
#[derive(Component)]
pub struct HealthBarMask {
    pub entity: Entity,
}

/// A component to identify the circular HP bar background circle for cleanup purposes
#[derive(Component)]
pub struct HealthBarBackground;

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

/// Marker component for boundary walls
#[derive(Component)]
pub struct BoundaryWall;

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
    pub charge_level: f32, // 0.0 = uncharged, 1.0 = fully charged
}

/// Marker component to indicate a projectile has already hit something (prevents multiple hits)
#[derive(Component)]
pub struct ProjectileHasHit;

/// Component to track shooting cooldown
#[derive(Component)]
pub struct Shooting {
    pub timer: f32,
}

/// Component to track charge shot charging state
#[derive(Component)]
pub struct ChargeShot {
    pub timer: f32,
    pub is_charging: bool,
}

/// Component to mark the visual charge effect (glow/particles around player)
#[derive(Component)]
pub struct ChargeEffect {
    pub player_entity: Entity,
}

/// Component to track invincibility frames (prevents damage spam)
#[derive(Component)]
pub struct Invincibility {
    pub timer: f32,
}

/// Component to track knockback effect (pushes player away when hit)
#[derive(Component)]
pub struct Knockback {
    pub velocity: Vec2,
    pub timer: f32,
}
