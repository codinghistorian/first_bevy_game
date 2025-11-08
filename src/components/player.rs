use bevy::prelude::*;

/// Marker component for the player character
#[derive(Component)]
pub struct Player;

/// Component to track player velocity (for jumping and gravity)
#[derive(Component)]
pub struct PlayerVelocity {
    pub y: f32,
    pub jump_type: JumpType,
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