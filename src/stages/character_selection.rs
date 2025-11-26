use crate::stages::game_menu::GameState;
use bevy::prelude::*;
use bevy::text::TextFont;

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

/// Spawns the character selection menu UI when entering the CharacterSelection state
pub fn spawn_character_selection_menu(mut commands: Commands) {
    use bevy::color::palettes::basic::{BLACK, WHITE};

    // Create two character boxes
    let breadman_entity = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                height: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(8.0)), // Thicker border for better visibility
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
                width: Val::Px(250.0),
                height: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(8.0)), // Thicker border for better visibility
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
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(40.0),
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
                    column_gap: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .add_child(breadman_entity)
                .add_child(cheeseman_entity);
        });
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

