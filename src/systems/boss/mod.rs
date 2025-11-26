//! Boss systems module.
//!
//! Handles boss behavior, including:
//! - Pattern loading and management (`pattern.rs`)
//! - Movement logic (`movement.rs`)
//! - Attack logic and projectile spawning (`attack.rs`)
//! - Spawning (`spawning.rs`)
//! - UI/Health bars (`ui.rs`)

pub mod attack;
pub mod movement;
pub mod pattern;
pub mod spawning;
pub mod ui;

pub use attack::*;
pub use movement::*;
pub use pattern::*;
pub use spawning::*;
pub use ui::*;
