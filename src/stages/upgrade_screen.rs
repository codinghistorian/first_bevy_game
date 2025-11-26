use crate::config::gameplay::HP_RESTORATION_AMOUNT;
use crate::stages::game_menu::{CurrentStage, DefeatedBoss, GameState};
use bevy::prelude::*;

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

/// Marker component for the stage upgrade screen UI root
#[derive(Component)]
pub struct StageUpgradeScreen;

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
            current_hp: 100.0,       // Start with base max HP
            defense_multiplier: 1.0, // Start with no defense bonus
            has_boss_weapon: false,
            boss_weapon_type: None,
        }
    }
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
                width: Val::Px(400.0),
                height: Val::Px(120.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(8.0)),
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
                TextColor(Color::WHITE.into()),
            ));
            parent.spawn((
                Text::new("+25 HP"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));
        })
        .id();

    let weapon_button_entity = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(400.0),
                height: Val::Px(120.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(8.0)),
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
                TextColor(Color::WHITE.into()),
            ));
            parent.spawn((
                Text::new("Use the defeated boss's weapon"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));
        })
        .id();

    // Create the root menu container
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
                TextColor(Color::WHITE.into()),
            ));

            // Instructions
            parent.spawn((
                Text::new("Choose an upgrade (Arrow Keys + Enter):"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE.into()),
            ));

            // Button container with the two upgrade options
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(20.0),
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
                player_upgrades.current_hp = (player_upgrades.current_hp
                    + HP_RESTORATION_AMOUNT)
                    .min(max_hp);
                info!(
                    "Selected upgrade: Restore HP (+{})",
                    HP_RESTORATION_AMOUNT
                );
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

