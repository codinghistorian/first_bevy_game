use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for boss entities
#[derive(Component)]
pub struct Boss;

/// Different types of bosses in the game
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BossType {
    /// Default/test boss
    Default,
    // Add more boss types here as you create them
    // FireMan,
    // IceMan,
    // etc.
}

impl Default for BossType {
    fn default() -> Self {
        BossType::Default
    }
}

/// Boss data structure containing all boss-specific information
#[derive(Component, Clone)]
pub struct BossData {
    /// The type of boss
    pub boss_type: BossType,
    /// Sprite/image handle for the boss
    pub sprite: Option<Handle<Image>>,
    /// Boss name
    pub name: String,
    /// Attack pattern configuration
    pub attack_pattern: AttackPattern,
    /// Movement pattern configuration
    pub movement_pattern: MovementPattern,
    /// Boss color (fallback if sprite not loaded)
    pub color: Color,
    /// Boss size
    pub size: Vec2,
}

impl Default for BossData {
    fn default() -> Self {
        Self {
            boss_type: BossType::Default,
            sprite: None,
            name: "Boss".to_string(),
            attack_pattern: AttackPattern::default(),
            movement_pattern: MovementPattern::default(),
            color: Color::srgb(0.8, 0.1, 0.1),
            size: Vec2::new(32.0, 64.0),
        }
    }
}

/// Vec2 configuration for JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vec2Config {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2Config> for Vec2 {
    fn from(v: Vec2Config) -> Self {
        Vec2::new(v.x, v.y)
    }
}

impl From<Vec2> for Vec2Config {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// Individual attack action in a sequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttackAction {
    pub action_type: String,           // "shoot", "wait", "burst", etc.
    pub direction: Option<Vec2Config>, // Direction to shoot
    pub count: Option<u32>,            // Number of shots
    pub delay: Option<f32>,            // Delay before next action
    pub spread: Option<f32>,           // Spread angle for multi-shot
}

/// Attack pattern types for bosses
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AttackPattern {
    /// No attacks
    None,
    /// Simple single shot
    SingleShot {
        cooldown: f32,
        projectile_speed: f32,
        #[serde(default)]
        cardinal_only: bool,
    },
    /// Triple shot pattern
    TripleShot {
        cooldown: f32,
        projectile_speed: f32,
        spread_angle: f32,
    },
    /// Rapid fire
    RapidFire {
        cooldown: f32,
        projectile_speed: f32,
        burst_count: u32,
        burst_delay: f32,
    },
    /// Pattern with multiple actions in sequence
    Sequence {
        actions: Vec<AttackAction>,
        loop_pattern: bool,
    },
    /// Custom pattern (extend as needed)
    Custom {
        cooldown: f32,
        // Add custom attack parameters here
    },
}

impl Default for AttackPattern {
    fn default() -> Self {
        AttackPattern::SingleShot {
            cooldown: 2.0,
            projectile_speed: 300.0,
            cardinal_only: false,
        }
    }
}

/// Movement pattern types for bosses
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MovementPattern {
    /// Stationary boss
    Stationary,
    /// Horizontal patrol between two points
    HorizontalPatrol {
        left_bound: f32,
        right_bound: f32,
        speed: f32,
    },
    /// Vertical movement
    VerticalPatrol {
        top_bound: f32,
        bottom_bound: f32,
        speed: f32,
    },
    /// Circular movement
    Circular {
        center: Vec2Config,
        radius: f32,
        speed: f32,
    },
    /// Pattern with multiple waypoints
    Waypoint {
        waypoints: Vec<Vec2Config>,
        speed: f32,
        loop_path: bool,
    },
    /// Custom movement (extend as needed)
    Custom,
}

impl Default for MovementPattern {
    fn default() -> Self {
        MovementPattern::Stationary
    }
}

/// Component to track boss attack state
#[derive(Component)]
pub struct BossAttackState {
    pub timer: f32,
    pub burst_count: u32,
    pub burst_timer: f32,
}

impl Default for BossAttackState {
    fn default() -> Self {
        Self {
            timer: 0.0,
            burst_count: 0,
            burst_timer: 0.0,
        }
    }
}

/// Component to track boss movement state
#[derive(Component)]
pub struct BossMovementState {
    pub direction: f32,     // -1.0 for left/up, 1.0 for right/down
    pub current_angle: f32, // For circular movement
}

impl Default for BossMovementState {
    fn default() -> Self {
        Self {
            direction: 1.0,
            current_angle: 0.0,
        }
    }
}

/// Resource to store boss configurations
/// This allows you to load boss data from files or define them in code
#[derive(Resource)]
pub struct BossRegistry {
    pub bosses: Vec<BossData>,
}

impl Default for BossRegistry {
    fn default() -> Self {
        Self {
            bosses: vec![
                // Default boss
                BossData {
                    boss_type: BossType::Default,
                    sprite: None,
                    name: "Default Boss".to_string(),
                    attack_pattern: AttackPattern::SingleShot {
                        cooldown: 2.0,
                        projectile_speed: 300.0,
                        cardinal_only: false,
                    },
                    movement_pattern: MovementPattern::Stationary,
                    color: Color::srgb(0.8, 0.1, 0.1),
                    size: Vec2::new(32.0, 64.0),
                },
                // Add more boss configurations here
            ],
        }
    }
}

/// Component to identify boss HP bar container nodes for cleanup
#[derive(Component)]
pub struct BossHealthBarContainer;

impl BossRegistry {
    /// Get boss data by type
    pub fn get_boss_data(&self, boss_type: BossType) -> Option<&BossData> {
        self.bosses.iter().find(|boss| boss.boss_type == boss_type)
    }
}

/// Component for boss projectiles
#[derive(Component)]
pub struct BossProjectile {
    pub damage: f32,
    pub speed: f32,
}
