use bevy::text::prelude::{TextColor, TextFont};
use bevy::{
    color::palettes::basic::{BLACK, WHITE},
    prelude::*,
};

/// Game state to manage transitions between character selection and gameplay
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, Component)]
pub enum GameState {
    #[default]
    CharacterSelection,
    InGame,
    StageUpgrade, // Intermediate stage between bosses for upgrades
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

/// Marker component for the stage upgrade screen UI root
#[derive(Component)]
pub struct StageUpgradeScreen;

/// Resource to track the current stage number (1-indexed)
#[derive(Resource, Default)]
pub struct CurrentStage(pub u32);

/// Component to identify upgrade option buttons
#[derive(Component)]
pub enum UpgradeButton {
    IncreaseHp,
    AcquireWeapon,
    ImproveDefense,
}

/// Resource to track which upgrade option is currently selected (0 = HP, 1 = Weapon, 2 = Defense)
#[derive(Resource, Default)]
pub struct SelectedUpgradeIndex(pub usize);

/// Resource to store which boss was defeated (for win screen display)
#[derive(Resource, Default)]
pub struct DefeatedBoss {
    pub boss_type: Option<crate::components::boss::BossType>,
}

/// Resource to track whether to show the win screen (only for final stage)
#[derive(Resource, Default)]
pub struct ShowWinScreen(pub bool);

/// Resource to track player upgrades and stats
#[derive(Resource)]
pub struct PlayerUpgrades {
    pub max_hp_bonus: f32,       // Additional HP added to base max HP
    pub defense_multiplier: f32, // Damage reduction (1.0 = no reduction, 0.5 = 50% less damage)
    pub has_boss_weapon: bool,   // Whether player has acquired boss weapon
    pub boss_weapon_type: Option<crate::components::boss::BossType>, // Which boss weapon was acquired
}

impl PlayerUpgrades {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for PlayerUpgrades {
    fn default() -> Self {
        Self {
            max_hp_bonus: 0.0,
            defense_multiplier: 1.0, // Start with no defense bonus
            has_boss_weapon: false,
            boss_weapon_type: None,
        }
    }
}

/// Spawns a UI camera for rendering
pub fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawns the character selection menu UI when entering the CharacterSelection state
pub fn spawn_character_selection_menu(mut commands: Commands) {
    // Create two character boxes
    let megaman_entity = commands
        .spawn((
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
        ))
        .with_children(|parent| {
            // Character name
            parent.spawn((
                Text::new("Megaman"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        })
        .id();

    let protoman_entity = commands
        .spawn((
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
        ))
        .with_children(|parent| {
            // Character name
            parent.spawn((
                Text::new("Protoman"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        })
        .id();

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
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(40.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .add_child(megaman_entity)
                .add_child(protoman_entity);
        });
}

/// Spawns the ingame 2D game scene when entering the InGame state
pub fn spawn_in_game_screen(mut commands: Commands) {
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
            parent
                .spawn((
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

/// Spawns the stage upgrade screen (intermediate screen between stages)
pub fn spawn_stage_upgrade_screen(
    mut commands: Commands,
    _defeated_boss: Res<DefeatedBoss>,
    _current_stage: Res<CurrentStage>,
) {
    // Create three upgrade option buttons
    let hp_button_entity = commands
        .spawn((
            Button,
            Node {
                width: px(400.0),
                height: px(120.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(10.0),
                padding: UiRect::all(px(20.0)),
                border: UiRect::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.5, 0.3)), // Green for HP
            BorderColor::all(Color::srgb(1.0, 0.9, 0.0)), // Start with glow (first option is default selected)
            UpgradeButton::IncreaseHp,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Increase Max HP"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
            parent.spawn((
                Text::new("+50 Max HP"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        })
        .id();

    let weapon_button_entity = commands
        .spawn((
            Button,
            Node {
                width: px(400.0),
                height: px(120.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(10.0),
                padding: UiRect::all(px(20.0)),
                border: UiRect::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.3, 0.5)), // Purple for weapon
            BorderColor::all(Color::srgb(0.4, 0.2, 0.4)), // Not selected
            UpgradeButton::AcquireWeapon,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Acquire Boss Weapon"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
            parent.spawn((
                Text::new("Use the defeated boss's weapon"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        })
        .id();

    let defense_button_entity = commands
        .spawn((
            Button,
            Node {
                width: px(400.0),
                height: px(120.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(10.0),
                padding: UiRect::all(px(20.0)),
                border: UiRect::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.5, 0.3)), // Yellow for defense
            BorderColor::all(Color::srgb(0.4, 0.4, 0.2)), // Not selected
            UpgradeButton::ImproveDefense,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Improve Defense"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
            parent.spawn((
                Text::new("Reduce damage taken by 25%"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        })
        .id();

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
            BackgroundColor(Color::srgb(0.2, 0.2, 0.3)), // Dark blue background
            StageUpgradeScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("STAGE CLEARED!"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));

            // Instructions
            parent.spawn((
                Text::new("Choose an upgrade (Arrow Keys + Enter):"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));

            // Button container with the three upgrade options
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(20.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .add_child(hp_button_entity)
                .add_child(weapon_button_entity)
                .add_child(defense_button_entity);
        });
}

/// Handles keyboard input for upgrade selection
pub fn handle_upgrade_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_index: ResMut<SelectedUpgradeIndex>,
    mut border_query: Query<(&UpgradeButton, &mut BorderColor)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut current_stage: ResMut<CurrentStage>,
    mut player_upgrades: ResMut<PlayerUpgrades>,
    defeated_boss: Res<DefeatedBoss>,
) {
    // Handle up/down arrow keys to navigate
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if selected_index.0 > 0 {
            selected_index.0 -= 1;
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        if selected_index.0 < 2 {
            selected_index.0 += 1;
        }
    }

    // Update border colors based on selection
    for (button, mut border_color) in &mut border_query {
        let is_selected = match button {
            UpgradeButton::IncreaseHp => selected_index.0 == 0,
            UpgradeButton::AcquireWeapon => selected_index.0 == 1,
            UpgradeButton::ImproveDefense => selected_index.0 == 2,
        };

        if is_selected {
            // Glowing border (bright yellow/gold)
            *border_color = BorderColor::all(Color::srgb(1.0, 0.9, 0.0));
        } else {
            // Normal border based on button type
            match button {
                UpgradeButton::IncreaseHp => {
                    *border_color = BorderColor::all(Color::srgb(0.2, 0.4, 0.2));
                }
                UpgradeButton::AcquireWeapon => {
                    *border_color = BorderColor::all(Color::srgb(0.4, 0.2, 0.4));
                }
                UpgradeButton::ImproveDefense => {
                    *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.2));
                }
            }
        }
    }

    // Handle Enter or Space to confirm selection
    if keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Space) {
        match selected_index.0 {
            0 => {
                // Increase HP
                player_upgrades.max_hp_bonus += 50.0;
                info!("Selected upgrade: Increase Max HP");
            }
            1 => {
                // Acquire boss weapon
                if let Some(boss_type) = defeated_boss.boss_type {
                    player_upgrades.has_boss_weapon = true;
                    player_upgrades.boss_weapon_type = Some(boss_type);
                }
                info!("Selected upgrade: Acquire Boss Weapon");
            }
            2 => {
                // Improve defense
                player_upgrades.defense_multiplier =
                    (player_upgrades.defense_multiplier - 0.25).max(0.0);
                info!("Selected upgrade: Improve Defense");
            }
            _ => {}
        }
        // Move to next stage
        current_stage.0 += 1;
        next_state.set(GameState::InGame);
    }
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
    use crate::systems::config::MAX_STAGES;

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

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCharacterIndex>()
            .init_resource::<SelectedUpgradeIndex>()
            .init_resource::<DefeatedBoss>()
            .init_resource::<ShowWinScreen>()
            .init_resource::<PlayerUpgrades>()
            .add_systems(Startup, spawn_ui_camera)
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
            .add_systems(OnEnter(GameState::InGame), spawn_in_game_screen)
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
