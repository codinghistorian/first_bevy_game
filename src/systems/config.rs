pub const SMALL_JUMP_CHARGE_RATIO: f32 = 0.43;

// Jump mechanics
pub const BASE_JUMP_STRENGTH: f32 = 400.0;
pub const BASE_GRAVITY: f32 = 800.0;

pub const HIGH_JUMP_STRENGTH_MULTIPLIER: f32 = 1.1;
pub const HIGH_JUMP_GRAVITY_MULTIPLIER: f32 = 1.1;

pub const SMALL_JUMP_STRENGTH_MULTIPLIER: f32 = 0.4;
pub const SMALL_JUMP_GRAVITY_MULTIPLIER: f32 = 1.2;

pub const MAX_CHARGE_TIME: f32 = 0.2;

// Knockback mechanics
pub const KNOCKBACK_FORCE: f32 = 700.0; // Initial force of knockback push (pixels per second)
pub const KNOCKBACK_DURATION: f32 = 0.7; // Duration of knockback effect (seconds)
pub const KNOCKBACK_DECAY_RATE: f32 = 0.9; // Velocity decay per frame (0.0-1.0, higher = slower decay)
pub const KNOCKBACK_MOVEMENT_REDUCTION: f32 = 0.3; // Player movement speed multiplier during knockback (0.0-1.0)

// Invincibility mechanics (damage immunity after taking damage)
pub const INVINCIBILITY_DURATION: f32 = 0.7; // Duration of invincibility after taking damage (seconds)

// Stage progression
pub const MAX_STAGES: u32 = 2; // Maximum number of stages in the game

// Knockback direction modifiers for different collision angles
pub const KNOCKBACK_TOP_HORIZONTAL_COMPONENT: f32 = 0.6; // Horizontal component when hitting from top (0.0-1.0)
pub const KNOCKBACK_TOP_VERTICAL_COMPONENT: f32 = 0.8; // Vertical component when hitting from top (0.0-1.0)
pub const KNOCKBACK_SIDE_VERTICAL_COMPONENT: f32 = 0.3; // Vertical component when hitting from side (0.0-1.0, adds slight upward push)