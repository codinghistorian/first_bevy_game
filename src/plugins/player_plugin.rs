use bevy::prelude::*;
use crate::stages::game_menu::{GameState, despawn_screen};
use crate::systems::player::{player_movement, spawn_player_and_level, player_shooting, projectile_movement, setup_player_hp_bar, update_health_bars, change_health, spawn_boss, setup_boss_hp_bar, player_boss_collision, projectile_boss_collision, apply_knockback, check_game_outcome};
use crate::components::player::{Player, Floor, Projectile, HealthBar};
use crate::components::boss::{Boss, BossRegistry};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<BossRegistry>()
            .add_systems(OnEnter(GameState::InGame), (spawn_player_and_level, spawn_boss))
            .add_systems(OnEnter(GameState::InGame), (setup_player_hp_bar, setup_boss_hp_bar).after(spawn_player_and_level).after(spawn_boss))
            .add_systems(
                Update,
                (
                    player_movement,
                    apply_knockback.after(player_movement), // Apply knockback after normal movement
                    player_shooting,
                    projectile_movement,
                    player_boss_collision,
                    projectile_boss_collision,
                    check_game_outcome, // Check for win/lose conditions
                    update_health_bars,
                    change_health,
                ).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (despawn_screen::<Player>, despawn_screen::<Boss>, despawn_screen::<Floor>, despawn_screen::<Projectile>, despawn_screen::<HealthBar>),
            );
    }
}
