use bevy::{color::palettes::basic::{WHITE, BLACK}, prelude::*};
use bevy::text::prelude::{TextFont, TextColor};

/// Game state to manage transitions between character selection and gameplay
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, Component)]
pub enum GameState {
    #[default]
    CharacterSelection,
    InGame,
    GameOver,
    GameWin,
}

/// Resource to store the currently selected character
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCharacter {
    Megaman,
    Protoman,
}

impl Default for SelectedCharacter {
    fn default() -> Self {
        SelectedCharacter::Megaman
    }
}

/// Component to mark character selection buttons
#[derive(Component)]
pub enum CharacterButton {
    Megaman,
    Protoman,
}

/// Resource to track which character box is currently selected (0 = Megaman, 1 = Protoman)
#[derive(Resource, Default)]
pub struct SelectedCharacterIndex(pub usize);

/// Marker component for the character selection menu UI root
#[derive(Component)]
pub struct CharacterSelectionMenu;

/// Marker component for the game over screen UI root
#[derive(Component)]
pub struct GameOverScreen;

/// Marker component for the game win screen UI root
#[derive(Component)]
pub struct GameWinScreen;

/// Resource to track the current stage number (1-indexed)
#[derive(Resource, Default)]
pub struct CurrentStage(pub u32);

/// Resource to store which boss was defeated (for win screen display)
#[derive(Resource, Default)]
pub struct DefeatedBoss {
    pub boss_type: Option<crate::components::boss::BossType>,
}

/// Resource to track whether to show the win screen (only for final stage)
#[derive(Resource, Default)]
pub struct ShowWinScreen(pub bool);



/// Spawns a UI camera for rendering
pub fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawns the character selection menu UI when entering the CharacterSelection state
pub fn spawn_character_selection_menu(mut commands: Commands) {
    // Create two character boxes
    let megaman_entity = commands.spawn((
        Button,
        Node {
            width: px(250.0),
            height: px(300.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(20.0),
            padding: UiRect::all(px(20.0)),
            border: UiRect::all(px(8.0)), // Thicker border for better visibility
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.4, 0.9)),
        BorderColor::all(Color::srgb(1.0, 0.8, 0.0)), // Start with glow (Megaman is default selected)
        CharacterButton::Megaman,
    )).with_children(|parent| {
        // Character name
        parent.spawn((
            Text::new("Megaman"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(WHITE.into()),
        ));
    }).id();

    let protoman_entity = commands.spawn((
        Button,
        Node {
            width: px(250.0),
            height: px(300.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(20.0),
            padding: UiRect::all(px(20.0)),
            border: UiRect::all(px(8.0)), // Thicker border for better visibility
            ..default()
        },
        BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
        BorderColor::all(Color::srgb(0.7, 0.1, 0.1)), // Not selected
        CharacterButton::Protoman,
    )).with_children(|parent| {
        // Character name
        parent.spawn((
            Text::new("Protoman"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(WHITE.into()),
        ));
    }).id();

    // Create the root menu container
    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(40.0),
                ..default()
            },
            BackgroundColor(WHITE.into()),
            CharacterSelectionMenu,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Select Your Character"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(BLACK.into()),
            ));

            // Button container with the two character boxes
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(40.0),
                align_items: AlignItems::Center,
                ..default()
            }).add_child(megaman_entity).add_child(protoman_entity);
        });
}

/// Spawns the ingame 2D game scene when entering the InGame state
pub fn spawn_in_game_screen(
    mut commands: Commands,
) {
    // Spawn game camera (separate from UI camera)
    commands.spawn(Camera2d);

    // Background color via clear color (black)
    commands.insert_resource(ClearColor(Color::BLACK));
}

/// Handles keyboard input for character selection
pub fn handle_keyboard_selection(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_index: ResMut<SelectedCharacterIndex>,
    mut border_query: Query<(&CharacterButton, &mut BorderColor)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected_character: ResMut<SelectedCharacter>,
) {
    // Handle left/right arrow keys to navigate
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        if selected_index.0 > 0 {
            selected_index.0 -= 1;
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        if selected_index.0 < 1 {
            selected_index.0 += 1;
        }
    }

    // Update border colors based on selection
    for (button, mut border_color) in &mut border_query {
        let is_selected = match button {
            CharacterButton::Megaman => selected_index.0 == 0,
            CharacterButton::Protoman => selected_index.0 == 1,
        };

        if is_selected {
            // Glowing border (bright yellow/gold)
            *border_color = BorderColor::all(Color::srgb(1.0, 0.9, 0.0));
        } else {
            // Normal border
            match button {
                CharacterButton::Megaman => {
                    *border_color = BorderColor::all(Color::srgb(0.1, 0.2, 0.7));
                }
                CharacterButton::Protoman => {
                    *border_color = BorderColor::all(Color::srgb(0.7, 0.1, 0.1));
                }
            }
        }
    }

    // Handle Enter or Space to confirm selection
    if keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Space) {
        match selected_index.0 {
            0 => {
                *selected_character = SelectedCharacter::Megaman;
                info!("Selected character: Megaman");
            }
            1 => {
                *selected_character = SelectedCharacter::Protoman;
                info!("Selected character: Protoman");
            }
            _ => {}
        }
        next_state.set(GameState::InGame);
    }
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

/// Spawns the game over screen (dark background, white text)
pub fn spawn_game_over_screen(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(40.0),
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
                TextColor(WHITE.into()),
            ));
            
            // Restart instruction
            parent.spawn((
                Text::new("Press SPACE or ENTER to restart"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        });
}

/// Spawns the game win screen (bright background, extensible for different bosses)
pub fn spawn_game_win_screen(
    mut commands: Commands,
    defeated_boss: Res<DefeatedBoss>,
) {
    // Determine background color and text based on defeated boss
    let (bg_color, win_text) = match defeated_boss.boss_type {
        Some(crate::components::boss::BossType::Default) => {
            (Color::srgb(0.3, 0.6, 0.9), "VICTORY!")
        }
        // Add more boss types here as you create them
        // Some(crate::components::boss::BossType::FireMan) => {
        //     (Color::srgb(0.9, 0.4, 0.2), "FIRE MAN DEFEATED!")
        // }
        None => {
            (Color::srgb(0.4, 0.8, 0.4), "VICTORY!")
        }
    };

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(40.0),
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
                TextColor(WHITE.into()),
            ));
            
            // Restart instruction
            parent.spawn((
                Text::new("Press SPACE or ENTER to play again"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
            
            // Placeholder for boss-specific content (images, text, etc.)
            // This can be extended later to show different content based on boss type
            parent.spawn((
                Node {
                    width: px(400.0),
                    height: px(200.0),
                    margin: UiRect::all(px(20.0)),
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
                    TextColor(WHITE.into()),
                ));
            });
        });
}

/// Handles input for game over and win screens (restart functionality)
/// Also handles automatic stage progression when player wins
pub fn handle_game_end_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_stage: ResMut<CurrentStage>,
) {
    use crate::systems::config::MAX_STAGES;
    
    // Check if we're in win screen and should progress to next stage
    // This runs every frame, so we check if we just entered GameWin
    // If there are more stages, automatically transition to next stage
    // Otherwise, wait for player input to restart
    
    // For now, we'll handle stage progression in a separate system
    // that runs on entering GameWin state
    
    if keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Space) {
        // Reset stage counter when restarting
        current_stage.0 = 0;
        // Restart game by going back to character selection
        next_state.set(GameState::CharacterSelection);
    }
}

/// System to handle stage progression when entering win screen
pub fn handle_stage_progression(
    mut current_stage: ResMut<CurrentStage>,
    mut next_state: ResMut<NextState<GameState>>,
    mut show_win_screen: ResMut<ShowWinScreen>,
) {
    use crate::systems::config::MAX_STAGES;
    
    // Check current stage BEFORE incrementing
    let current_stage_num = current_stage.0;
    
    // If we're not at the final stage, automatically progress to next stage
    if current_stage_num < MAX_STAGES {
        // Move to next stage
        current_stage.0 += 1;
        // Don't show win screen - we're progressing to next stage
        show_win_screen.0 = false;
        // Transition back to InGame to load the next boss
        // This will trigger OnExit(GameWin) and OnEnter(InGame), properly reloading everything
        next_state.set(GameState::InGame);
    } else {
        // Final stage completed - show win screen
        show_win_screen.0 = true;
    }
}

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCharacterIndex>()
            .init_resource::<DefeatedBoss>()
            .init_resource::<ShowWinScreen>()
            .add_systems(Startup, spawn_ui_camera)
            .add_systems(OnEnter(GameState::CharacterSelection), spawn_character_selection_menu)
            .add_systems(
                Update,
                handle_keyboard_selection.run_if(in_state(GameState::CharacterSelection)),
            )
            .add_systems(
                OnExit(GameState::CharacterSelection),
                despawn_screen::<CharacterSelectionMenu>,
            )
            .add_systems(OnEnter(GameState::InGame), spawn_in_game_screen)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
            .add_systems(OnEnter(GameState::GameWin), (
                handle_stage_progression, // Check and progress stage FIRST (before showing win screen)
                spawn_game_win_screen.run_if(|show_win: Res<ShowWinScreen>| show_win.0),
            ))
            .add_systems(
                Update,
                (
                    handle_game_end_input.run_if(in_state(GameState::GameOver)),
                    handle_game_end_input.run_if(in_state(GameState::GameWin)),
                ),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                despawn_screen::<GameOverScreen>,
            )
            .add_systems(
                OnExit(GameState::GameWin),
                despawn_screen::<GameWinScreen>,
            );
    }
}