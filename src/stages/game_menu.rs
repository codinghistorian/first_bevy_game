//! Game stages and menu systems.
//!
//! This module coordinates the different game states and screens.
//! The plugin definition resides here.

pub use crate::stages::background::*;
pub use crate::stages::character_selection::*;
pub use crate::stages::game_over::*;
pub use crate::stages::opening_crawl::*;
pub use crate::stages::upgrade_screen::*;

use bevy::prelude::*;

/// Game state to manage transitions between character selection and gameplay
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, Component)]
pub enum GameState {
    #[default]
    OpeningCrawl,
    CharacterSelection,
    InGame,
    StageUpgrade, // Intermediate stage between bosses for upgrades
    GameOver,
    GameWin,
}

/// Resource to track the current stage number (1-indexed)
#[derive(Resource, Default)]
pub struct CurrentStage(pub u32);

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCharacterIndex>()
            .init_resource::<SelectedUpgradeIndex>()
            .init_resource::<DefeatedBoss>()
            .init_resource::<ShowWinScreen>()
            .init_resource::<PlayerUpgrades>()
            .init_resource::<BackgroundImages>()
            .add_systems(Startup, (spawn_ui_camera, load_background_images))
            .add_systems(
                Update,
                filter_loaded_background_images.run_if(resource_exists::<BackgroundImages>),
            )
            .add_systems(
                OnEnter(GameState::OpeningCrawl),
                spawn_opening_crawl,
            )
            .add_systems(
                Update,
                (
                    animate_crawl_text.run_if(in_state(GameState::OpeningCrawl)),
                    handle_opening_crawl_input.run_if(in_state(GameState::OpeningCrawl)),
                ),
            )
            .add_systems(
                OnExit(GameState::OpeningCrawl),
                (
                    despawn_screen::<OpeningCrawlScreen>,
                    |mut commands: Commands,
                     star_query: Query<Entity, With<Star>>,
                     camera_query: Query<Entity, (With<Camera3d>, Without<UiCamera>)>| {
                        // Despawn all stars
                        for star_entity in star_query.iter() {
                            commands.entity(star_entity).despawn();
                        }
                        // Despawn the 3D camera used for crawl
                        for camera_entity in camera_query.iter() {
                            commands.entity(camera_entity).despawn();
                        }
                    },
                ),
            )
            .add_systems(
                OnEnter(GameState::CharacterSelection),
                spawn_character_selection_menu,
            )
            .add_systems(
                Update,
                handle_keyboard_selection.run_if(in_state(GameState::CharacterSelection)),
            )
            .add_systems(
                OnExit(GameState::CharacterSelection),
                despawn_screen::<CharacterSelectionMenu>,
            )
            .add_systems(
                OnEnter(GameState::InGame),
                (despawn_ui_camera, spawn_in_game_screen),
            )
            .add_systems(
                Update,
                (animate_background).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), spawn_ui_camera)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
            .add_systems(
                OnEnter(GameState::StageUpgrade),
                (
                    |mut selected_index: ResMut<SelectedUpgradeIndex>| {
                        // Reset to first option when entering upgrade screen
                        selected_index.0 = 0;
                    },
                    spawn_stage_upgrade_screen,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameState::GameWin),
                (
                    handle_stage_progression, // Check and progress stage FIRST (before showing win screen)
                    spawn_game_win_screen.run_if(|show_win: Res<ShowWinScreen>| show_win.0),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_upgrade_input.run_if(in_state(GameState::StageUpgrade)),
                    handle_game_end_input.run_if(in_state(GameState::GameOver)),
                    handle_game_end_input.run_if(in_state(GameState::GameWin)),
                ),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                despawn_screen::<GameOverScreen>,
            )
            .add_systems(OnExit(GameState::GameWin), despawn_screen::<GameWinScreen>)
            .add_systems(
                OnExit(GameState::StageUpgrade),
                despawn_screen::<StageUpgradeScreen>,
            );
    }
}
