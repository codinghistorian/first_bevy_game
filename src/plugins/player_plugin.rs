use bevy::prelude::*;
use crate::stages::game_menu::{GameState, despawn_screen};
use crate::systems::player::{player_movement, spawn_player_and_level, player_shooting, projectile_movement, setup_player_hp_bar, update_health_bars, change_health};
use crate::components::player::{Player, Floor, Projectile, HealthBar};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), spawn_player_and_level)
            .add_systems(OnEnter(GameState::InGame), setup_player_hp_bar.after(spawn_player_and_level))
            .add_systems(
                Update,
                (
                    player_movement,
                    player_shooting,
                    projectile_movement,
                    update_health_bars,
                    change_health,
                ).run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (despawn_screen::<Player>, despawn_screen::<Floor>, despawn_screen::<Projectile>, despawn_screen::<HealthBar>),
            );
    }
}
