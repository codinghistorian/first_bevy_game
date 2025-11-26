// Player movement constants
pub const SMALL_JUMP_CHARGE_RATIO: f32 = 0.43;

pub const BASE_JUMP_STRENGTH: f32 = 400.0;
pub const BASE_GRAVITY: f32 = 800.0;

pub const HIGH_JUMP_STRENGTH_MULTIPLIER: f32 = 1.1;
pub const HIGH_JUMP_GRAVITY_MULTIPLIER: f32 = 1.1;

pub const SMALL_JUMP_STRENGTH_MULTIPLIER: f32 = 0.4;
pub const SMALL_JUMP_GRAVITY_MULTIPLIER: f32 = 1.2;

pub const MAX_CHARGE_TIME: f32 = 0.2; // For jump charge

// Charge shot mechanics
pub const CHARGE_SHOT_MAX_TIME: f32 = 1.0; // Maximum charge time in seconds
pub const CHARGE_SHOT_MIN_TIME: f32 = 0.1; // Minimum charge time to fire a charged shot
pub const CHARGE_SHOT_COOLDOWN: f32 = 0.3; // Cooldown after firing a charged shot
pub const CHARGE_SHOT_STRONG_THRESHOLD: f32 = 0.95; // Charge ratio required for strongest shot bonuses
pub const NORMAL_SHOT_COOLDOWN: f32 = 0.5; // Cooldown for normal (quick tap) shots

