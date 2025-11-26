use crate::config::gameplay::MAX_STAGES;
use crate::stages::game_menu::{CurrentStage, GameState, PlayerUpgrades};
use bevy::prelude::*;

/// Marker component for the game over screen UI root
#[derive(Component)]
pub struct GameOverScreen;

/// Marker component for the game win screen UI root
#[derive(Component)]
pub struct GameWinScreen;

/// Resource to store which boss was defeated (for win screen display)
#[derive(Resource, Default)]
pub struct DefeatedBoss {
    pub boss_type: Option<crate::components::boss::BossType>,
}

/// Resource to track whether to show the win screen (only for final stage)
#[derive(Resource, Default)]
pub struct ShowWinScreen(pub bool);

/// Spawns the game over screen (dark background, white text)
pub fn spawn_game_over_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)), // Dark background
            GameOverScreen,
        ))
        .with_children(|parent| {
            // Game Over text
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));

            // Restart instruction
            parent.spawn((
                Text::new("Press SPACE or ENTER to restart"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));
        });
}

/// Spawns the game win screen (bright background, extensible for different bosses)
pub fn spawn_game_win_screen(mut commands: Commands, defeated_boss: Res<DefeatedBoss>) {
    // Determine background color and text based on defeated boss
    let (bg_color, win_text) = match defeated_boss.boss_type {
        Some(crate::components::boss::BossType::Default) => {
            (Color::srgb(0.3, 0.6, 0.9), "VICTORY!")
        }
        // Add more boss types here as you create them
        // Some(crate::components::boss::BossType::FireMan) => {
        //     (Color::srgb(0.9, 0.4, 0.2), "FIRE MAN DEFEATED!")
        // }
        None => (Color::srgb(0.4, 0.8, 0.4), "VICTORY!"),
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(bg_color),
            GameWinScreen,
        ))
        .with_children(|parent| {
            // Victory text
            parent.spawn((
                Text::new(win_text),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));

            // Restart instruction
            parent.spawn((
                Text::new("Press SPACE or ENTER to play again"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));

            // Placeholder for boss-specific content (images, text, etc.)
            // This can be extended later to show different content based on boss type
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(200.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|content_parent| {
                    content_parent.spawn((
                        Text::new("Boss-specific content area\n(Add images/text here)"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE.into()),
                    ));
                });
        });
}

/// Handles input for game over and win screens (restart functionality)
pub fn handle_game_end_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_stage: ResMut<CurrentStage>,
    mut player_upgrades: ResMut<PlayerUpgrades>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Space) {
        // Reset stage counter and upgrades when restarting
        current_stage.0 = 0;
        *player_upgrades = PlayerUpgrades::new();
        // Restart game by going back to character selection
        next_state.set(GameState::CharacterSelection);
    }
}

/// System to handle stage progression when entering win screen
pub fn handle_stage_progression(
    current_stage: Res<CurrentStage>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show_win_screen: ResMut<ShowWinScreen>,
) {
    // Check current stage BEFORE incrementing
    let current_stage_num = current_stage.0;

    // If we're not at the final stage, go to upgrade screen
    if current_stage_num < MAX_STAGES {
        // Don't show win screen - we're going to upgrade screen
        show_win_screen.0 = false;
        // Transition to upgrade screen
        next_state.set(GameState::StageUpgrade);
    } else {
        // Final stage completed - show win screen
        show_win_screen.0 = true;
    }
}

