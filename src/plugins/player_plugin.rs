use crate::components::boss::{Boss, BossHealthBarContainer, BossRegistry};
use crate::components::player::{BoundaryWall, ChargeEffect, Floor, HealthBar, HealthBarBackground, HealthBarMask, Player, Projectile};
use crate::stages::game_menu::{BackgroundImage, CurrentStage, GameState, PlayerUpgrades, despawn_screen};
use crate::systems::boss::{
    BossPatternRegistry, BossProjectile, boss_attacks, boss_movement, boss_projectile_movement,
    boss_projectile_player_collision, load_stage_boss_pattern, setup_boss_hp_bar,
};
use crate::systems::boundaries::spawn_boundaries;
use crate::systems::player::{
    animate_charge_effect, apply_knockback, change_health, check_game_outcome, manage_charge_effect,
    persist_player_hp, player_boss_collision, player_movement, player_shooting, projectile_boss_collision,
    projectile_movement, setup_player_hp_bar, spawn_boss, spawn_player_and_level, update_health_bars,
};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BossRegistry>()
            .init_resource::<BossPatternRegistry>()
            .init_resource::<CurrentStage>()
            .init_resource::<PlayerUpgrades>()
            .add_systems(
                OnEnter(GameState::InGame),
                (
                    // Initialize stage to 1 only if starting fresh (stage is 0)
                    |mut stage: ResMut<CurrentStage>| {
                        if stage.0 == 0 {
                            stage.0 = 1;
                        }
                    },
                    // Load boss pattern for current stage
                    load_stage_boss_pattern,
                    // Spawn player, boss, and boundaries
                    spawn_player_and_level,
                    spawn_boss,
                    spawn_boundaries,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameState::InGame),
                (setup_player_hp_bar, setup_boss_hp_bar)
                    .after(spawn_player_and_level)
                    .after(spawn_boss),
            )
            .add_systems(
                Update,
                (
                    player_movement,
                    apply_knockback.after(player_movement), // Apply knockback after normal movement
                    player_shooting,
                    manage_charge_effect.after(player_shooting), // Manage charge effect spawn/despawn
                    animate_charge_effect.after(manage_charge_effect), // Animate charge effect
                    projectile_movement,
                    boss_movement,            // Boss movement system
                    boss_attacks,             // Boss attack system
                    boss_projectile_movement, // Boss projectile movement
                    boss_projectile_player_collision.after(boss_projectile_movement), // Boss projectile hits player (after movement)
                    player_boss_collision,
                    projectile_boss_collision,
                    persist_player_hp, // Persist player HP to upgrades resource
                    check_game_outcome, // Check for win/lose conditions
                    update_health_bars,
                    change_health,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (
                    persist_player_hp, // Save HP before despawning
                    despawn_screen::<Player>,
                    despawn_screen::<Boss>,
                    despawn_screen::<Floor>,
                    despawn_screen::<Projectile>,
                    despawn_screen::<HealthBar>,
                    despawn_screen::<HealthBarBackground>,
                    despawn_screen::<HealthBarMask>,
                    despawn_screen::<BossHealthBarContainer>,
                    despawn_screen::<BackgroundImage>,
                    despawn_screen::<BossProjectile>,
                    despawn_screen::<BoundaryWall>,
                    despawn_screen::<ChargeEffect>,
                ),
            );
    }
}
