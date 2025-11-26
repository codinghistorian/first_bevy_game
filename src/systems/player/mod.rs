//! Player systems module.
//!
//! Handles player mechanics, including:
//! - Spawning and level setup (`spawning.rs`)
//! - Movement and physics (`movement.rs`)
//! - Shooting and projectiles (`shooting.rs`)
//! - Collision detection (`collision.rs`)
//! - Health and UI (`health.rs`)
//! - Animation (`animation.rs`)

pub mod animation;
pub mod collision;
pub mod health;
pub mod movement;
pub mod shooting;
pub mod spawning;

pub use animation::*;
pub use collision::*;
pub use health::*;
pub use movement::*;
pub use shooting::*;
pub use spawning::*;
