use bevy::text::prelude::{TextColor, TextFont};
use bevy::{
    color::palettes::basic::{BLACK, WHITE},
    prelude::*,
    sprite::Anchor,
};
use crate::systems::config::{BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, BOUNDARY_BOTTOM, BACKGROUND_PADDING};

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
    Breadman,
    Cheeseman,
}

impl Default for SelectedCharacter {
    fn default() -> Self {
        SelectedCharacter::Breadman
    }
}

/// Component to mark character selection buttons
#[derive(Component)]
pub enum CharacterButton {
    Breadman,
    Cheeseman,
}

/// Resource to track which character box is currently selected (0 = Breadman, 1 = Cheeseman)
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

/// Resource to track which upgrade option is currently selected (0 = HP, 1 = Weapon)
#[derive(Resource, Default)]
pub struct SelectedUpgradeIndex(pub usize);

/// Component for background images
#[derive(Component)]
pub struct BackgroundImage;

/// Component to identify the UI camera
#[derive(Component)]
pub struct UiCamera;

/// Resource to hold background image handles for each stage
#[derive(Resource, Default)]
pub struct BackgroundImages {
    pub stage_1: Vec<Handle<Image>>,
}

impl BackgroundImages {
    pub fn get_stage_images(&self, stage: u32) -> Option<&Vec<Handle<Image>>> {
        match stage {
            1 => Some(&self.stage_1),
            _ => None,
        }
    }
}

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
    pub current_hp: f32,         // Current HP that persists between stages
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
            current_hp: 100.0, // Start with base max HP
            defense_multiplier: 1.0, // Start with no defense bonus
            has_boss_weapon: false,
            boss_weapon_type: None,
        }
    }
}

/// Loads background images for each stage dynamically by iterating through available images
pub fn load_background_images(mut background_images: ResMut<BackgroundImages>, asset_server: Res<AssetServer>) {
    info!("Loading background images for stage 1...");
    
    // Maximum number of images to check (adjust if you have more than 30 images)
    const MAX_IMAGES: u32 = 30;
    
    // Iterate through all possible image numbers and load them
    let mut handles = Vec::new();
    for i in 1..=MAX_IMAGES {
        let image_path = format!("images/backgrounds/stage_1/stage_1_{}.jpg", i);
        handles.push(asset_server.load(image_path));
    }
    
    background_images.stage_1 = handles;
    info!("Attempted to load up to {} background images for stage 1", MAX_IMAGES);
    info!("Loaded {} background image handles for stage 1", background_images.stage_1.len());
    for (i, handle) in background_images.stage_1.iter().enumerate() {
        info!("Stage 1 image {}: handle id = {:?}", i + 1, handle.id());
    }
}

/// Filters out background image handles that failed to load (removes blank images)
/// Uses a timer to wait a bit before filtering to give assets time to load/fail
pub fn filter_loaded_background_images(
    mut background_images: ResMut<BackgroundImages>,
    mut timer: Local<Option<f32>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    // Wait 0.5 seconds before filtering to give assets time to load/fail
    let wait_time = 0.5;
    
    let elapsed = timer.get_or_insert(0.0);
    *elapsed += time.delta_secs();
    
    if *elapsed < wait_time {
        return;
    }
    
    // Only filter once
    if *elapsed >= wait_time + 0.1 {
        return;
    }

    // Filter stage_1 images to only include successfully loaded ones
    let mut valid_handles = Vec::new();
    for handle in background_images.stage_1.iter() {
        let load_state = asset_server.load_state(handle);
        // Only keep handles that are fully loaded (not loading, not failed)
        if matches!(load_state, bevy::asset::LoadState::Loaded) {
            valid_handles.push(handle.clone());
        }
    }
    
    // Only update if we found valid images and the count is different
    if !valid_handles.is_empty() && valid_handles.len() != background_images.stage_1.len() {
        info!(
            "Filtered background images: {} valid out of {} total",
            valid_handles.len(),
            background_images.stage_1.len()
        );
        background_images.stage_1 = valid_handles;
    }
}

/// Animates background images by cycling through frames
pub fn animate_background(
    time: Res<Time>,
    mut timer: Local<f32>,
    background_images: Res<BackgroundImages>,
    current_stage: Res<CurrentStage>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Sprite, With<BackgroundImage>>,
) {
    // Only animate if we have background images for this stage
    if let Some(images) = background_images.get_stage_images(current_stage.0) {
        if images.is_empty() {
            return;
        }

        // Filter to only fully loaded images
        let loaded_images: Vec<_> = images
            .iter()
            .filter(|handle| {
                matches!(
                    asset_server.load_state(&**handle),
                    bevy::asset::LoadState::Loaded
                )
            })
            .cloned()
            .collect();

        if loaded_images.is_empty() {
            return;
        }

        // Update timer
        *timer += time.delta_secs();

        // Change frame every 2.0 seconds for smoother animation
        if *timer >= 2.0 {
            *timer = 0.0;

            // Cycle through background images
            for mut sprite in query.iter_mut() {
                // Find current image index in the loaded images list
                let current_index = loaded_images
                    .iter()
                    .position(|handle| handle.id() == sprite.image.id());
                if let Some(current_index) = current_index {
                    let next_index = (current_index + 1) % loaded_images.len();
                    sprite.image = loaded_images[next_index].clone();
                } else {
                    // If current image isn't in loaded list, switch to first loaded image
                    sprite.image = loaded_images[0].clone();
                }
            }
        }
    }
}

/// Despawns the UI camera (used when entering gameplay)
pub fn despawn_ui_camera(mut commands: Commands, ui_camera_query: Query<Entity, With<UiCamera>>) {
    for entity in ui_camera_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Spawns a UI camera for rendering
pub fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // UI camera renders on top
            ..default()
        },
        UiCamera,
    ));
}

/// Spawns the character selection menu UI when entering the CharacterSelection state
pub fn spawn_character_selection_menu(mut commands: Commands) {
    // Create two character boxes
    let breadman_entity = commands
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
            BorderColor::all(Color::srgb(1.0, 0.8, 0.0)), // Start with glow (Breadman is default selected)
            CharacterButton::Breadman,
        ))
        .with_children(|parent| {
            // Character name
            parent.spawn((
                Text::new("Breadman"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        })
        .id();

    let cheeseman_entity = commands
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
            CharacterButton::Cheeseman,
        ))
        .with_children(|parent| {
            // Character name
            parent.spawn((
                Text::new("Cheeseman"),
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
                .add_child(breadman_entity)
                .add_child(cheeseman_entity);
        });
}

/// Spawns the ingame 2D game scene when entering the InGame state
pub fn spawn_in_game_screen(
    mut commands: Commands,
    background_images: Res<BackgroundImages>,
    mut current_stage: ResMut<CurrentStage>,
    asset_server: Res<AssetServer>,
) {
    // Spawn game camera (separate from UI camera) - use Camera2dBundle as recommended
    commands.spawn((
        Camera2d,
        Camera {
            order: 0, // Game camera renders first (background)
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Ensure the first gameplay entry starts at stage 1
    if current_stage.0 == 0 {
        current_stage.0 = 1;
    }

    let stage_number = current_stage.0;

    // Spawn background image for current stage if available
    if let Some(image_handles) = background_images.get_stage_images(stage_number) {
        if !image_handles.is_empty() {
            info!(
                "Spawning background for stage {} with {} images",
                stage_number,
                image_handles.len()
            );
            
            // Get the first image handle
            let first_handle = &image_handles[0];
            let load_state = asset_server.load_state(first_handle);
            info!("First background image load state: {:?}, handle id: {:?}", load_state, first_handle.id());
            
            // Calculate background size to be slightly larger than game boundaries
            let bg_width = (BOUNDARY_RIGHT - BOUNDARY_LEFT) + (BACKGROUND_PADDING * 2.0);
            let bg_height = (BOUNDARY_TOP - BOUNDARY_BOTTOM) + (BACKGROUND_PADDING * 2.0);
            let bg_center_x = (BOUNDARY_LEFT + BOUNDARY_RIGHT) / 2.0;
            let bg_center_y = (BOUNDARY_BOTTOM + BOUNDARY_TOP) / 2.0;
            
            info!(
                "Background size: {}x{}, center: ({}, {})",
                bg_width, bg_height, bg_center_x, bg_center_y
            );
            
            // Spawn background sprite - ensure all required components are present
            commands.spawn((
                Sprite {
                    image: first_handle.clone(),
                    custom_size: Some(Vec2::new(bg_width, bg_height)),
                    ..default()
                },
                Anchor::CENTER,
                Transform::from_xyz(bg_center_x, bg_center_y, -10.0),
                GlobalTransform::default(),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
                BackgroundImage,
            ));
        } else {
            warn!("No background images available for stage {}", current_stage.0);
            commands.insert_resource(ClearColor(Color::BLACK));
        }
    } else {
        info!("No background images configured for stage {}", current_stage.0);
        commands.insert_resource(ClearColor(Color::BLACK));
    }
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
            CharacterButton::Breadman => selected_index.0 == 0,
            CharacterButton::Cheeseman => selected_index.0 == 1,
        };

        if is_selected {
            // Glowing border (bright yellow/gold)
            *border_color = BorderColor::all(Color::srgb(1.0, 0.9, 0.0));
        } else {
            // Normal border
            match button {
                CharacterButton::Breadman => {
                    *border_color = BorderColor::all(Color::srgb(0.1, 0.2, 0.7));
                }
                CharacterButton::Cheeseman => {
                    *border_color = BorderColor::all(Color::srgb(0.7, 0.1, 0.1));
                }
            }
        }
    }

    // Handle Enter or Space to confirm selection
    if keyboard_input.just_pressed(KeyCode::Enter) || keyboard_input.just_pressed(KeyCode::Space) {
        match selected_index.0 {
            0 => {
                *selected_character = SelectedCharacter::Breadman;
                info!("Selected character: Breadman");
            }
            1 => {
                *selected_character = SelectedCharacter::Cheeseman;
                info!("Selected character: Cheeseman");
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
                Text::new("Restore HP"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
            parent.spawn((
                Text::new("+25 HP"),
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

            // Button container with the two upgrade options
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(20.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .add_child(hp_button_entity)
                .add_child(weapon_button_entity);
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
        if selected_index.0 < 1 {
            selected_index.0 += 1;
        }
    }

    // Update border colors based on selection
    for (button, mut border_color) in &mut border_query {
        let is_selected = match button {
            UpgradeButton::IncreaseHp => selected_index.0 == 0,
            UpgradeButton::AcquireWeapon => selected_index.0 == 1,
            UpgradeButton::ImproveDefense => false, // Not used anymore
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
                // Restore HP
                let max_hp = 100.0 + player_upgrades.max_hp_bonus;
                player_upgrades.current_hp = (player_upgrades.current_hp + crate::systems::config::HP_RESTORATION_AMOUNT).min(max_hp);
                info!("Selected upgrade: Restore HP (+{})", crate::systems::config::HP_RESTORATION_AMOUNT);
            }
            1 => {
                // Acquire boss weapon
                if let Some(boss_type) = defeated_boss.boss_type {
                    player_upgrades.has_boss_weapon = true;
                    player_upgrades.boss_weapon_type = Some(boss_type);
                }
                info!("Selected upgrade: Acquire Boss Weapon");
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
            .init_resource::<BackgroundImages>()
            .add_systems(Startup, (spawn_ui_camera, load_background_images))
            .add_systems(
                Update,
                filter_loaded_background_images.run_if(resource_exists::<BackgroundImages>),
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
            .add_systems(OnEnter(GameState::InGame), (despawn_ui_camera, spawn_in_game_screen))
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
