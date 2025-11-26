// Gameplay mechanics (Knockback, Invincibility, Stages, Boundaries)

// Knockback mechanics
pub const KNOCKBACK_FORCE: f32 = 700.0; // Initial force of knockback push (pixels per second)
pub const KNOCKBACK_DURATION: f32 = 0.7; // Duration of knockback effect (seconds)
pub const KNOCKBACK_DECAY_RATE: f32 = 0.9; // Velocity decay per frame (0.0-1.0, higher = slower decay)
pub const KNOCKBACK_MOVEMENT_REDUCTION: f32 = 0.3; // Player movement speed multiplier during knockback (0.0-1.0)

// Knockback direction modifiers for different collision angles
pub const KNOCKBACK_TOP_HORIZONTAL_COMPONENT: f32 = 0.6; // Horizontal component when hitting from top (0.0-1.0)
pub const KNOCKBACK_TOP_VERTICAL_COMPONENT: f32 = 0.8; // Vertical component when hitting from top (0.0-1.0)
pub const KNOCKBACK_SIDE_VERTICAL_COMPONENT: f32 = 0.3; // Vertical component when hitting from side (0.0-1.0, adds slight upward push)

// Invincibility mechanics
pub const INVINCIBILITY_DURATION: f32 = 0.7; // Duration of invincibility after taking damage (seconds)

// Damage values (Player -> Boss)
pub const PLAYER_PROJECTILE_DAMAGE: f32 = 20.0; // Base damage dealt by player projectiles to boss
pub const CHARGE_SHOT_DAMAGE_MULTIPLIER: f32 = 3.0; // Fully charged shot deals 3x base damage

// Stage progression
pub const MAX_STAGES: u32 = 2; // Maximum number of stages in the game

// Upgrade values
pub const HP_RESTORATION_AMOUNT: f32 = 25.0; // Amount of HP restored when choosing HP upgrade

// Game boundaries
pub const BOUNDARY_LEFT: f32 = -350.0; // Left boundary X position
pub const BOUNDARY_RIGHT: f32 = 350.0; // Right boundary X position
pub const BOUNDARY_TOP: f32 = 200.0; // Top boundary Y position
pub const BOUNDARY_BOTTOM: f32 = -198.0; // Bottom boundary Y position (player ground level)
pub const BOUNDARY_WALL_THICKNESS: f32 = 4.0; // Thickness of boundary wall lines
pub const BACKGROUND_PADDING: f32 = 50.0; // Padding around boundaries for background image

