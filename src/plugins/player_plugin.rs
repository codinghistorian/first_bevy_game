use bevy::prelude::*;
use crate::stages::game_menu::{GameState, despawn_screen, CurrentStage, PlayerUpgrades};
use crate::systems::player::{player_movement, spawn_player_and_level, player_shooting, projectile_movement, setup_player_hp_bar, update_health_bars, change_health, spawn_boss, player_boss_collision, projectile_boss_collision, apply_knockback, check_game_outcome};
use crate::systems::boss::{boss_movement, boss_attacks, boss_projectile_movement, boss_projectile_player_collision, BossPatternRegistry, BossProjectile, load_stage_boss_pattern, setup_boss_hp_bar};
use crate::systems::boundaries::spawn_boundaries;
use crate::components::player::{Player, Floor, Projectile, HealthBar, BoundaryWall};
use crate::components::boss::{Boss, BossRegistry};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BossRegistry>()
            .init_resource::<BossPatternRegistry>()
            .init_resource::<CurrentStage>()
            .init_resource::<PlayerUpgrades>()
            .add_systems(OnEnter(GameState::InGame), (
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
            ).chain())
            .add_systems(OnEnter(GameState::InGame), (setup_player_hp_bar, setup_boss_hp_bar).after(spawn_player_and_level).after(spawn_boss))
            .add_systems(
                Update,
                (
                    player_movement,
                    apply_knockback.after(player_movement), // Apply knockback after normal movement
                    player_shooting,
                    projectile_movement,
                    boss_movement, // Boss movement system
                    boss_attacks, // Boss attack system
                    boss_projectile_movement, // Boss projectile movement
                    boss_projectile_player_collision, // Boss projectile hits player
                    player_boss_collision,
                    projectile_boss_collision,
                    check_game_outcome, // Check for win/lose conditions
                    update_health_bars,
                    change_health,
                ).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (despawn_screen::<Player>, despawn_screen::<Boss>, despawn_screen::<Floor>, despawn_screen::<Projectile>, despawn_screen::<HealthBar>, despawn_screen::<BossProjectile>, despawn_screen::<BoundaryWall>),
            );
    }
}
