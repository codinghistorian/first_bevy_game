use bevy::{color::palettes::basic::{WHITE, BLACK}, prelude::*};
use bevy::text::prelude::{TextFont, TextColor};

/// Game state to manage transitions between character selection and gameplay
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States, Component)]
pub enum GameState {
    #[default]
    CharacterSelection,
    InGame,
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

/// Marker component for the character selection menu UI root
#[derive(Component)]
pub struct CharacterSelectionMenu;

/// Marker component for the ingame screen UI root
#[derive(Component)]
pub struct InGameScreen;

/// Spawns the character selection menu UI when entering the CharacterSelection state
pub fn spawn_character_selection_menu(mut commands: Commands) {
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

            // Button container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(40.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Megaman colored box
                    parent
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
                                border: UiRect::all(px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.9)),
                            BorderColor::all(Color::srgb(0.1, 0.2, 0.7)),
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
                            // Visual box indicator
                            parent.spawn((
                                Node {
                                    width: px(180.0),
                                    height: px(180.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.5, 1.0)),
                                BorderColor::all(Color::srgb(0.1, 0.2, 0.7)),
                            ));
                        });

                    // Protoman colored box
                    parent
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
                                border: UiRect::all(px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
                            BorderColor::all(Color::srgb(0.7, 0.1, 0.1)),
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
                            // Visual box indicator
                            parent.spawn((
                                Node {
                                    width: px(180.0),
                                    height: px(180.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(1.0, 0.3, 0.3)),
                                BorderColor::all(Color::srgb(0.7, 0.1, 0.1)),
                            ));
                        });
                });
        });
}

/// Spawns the ingame screen UI when entering the InGame state
pub fn spawn_in_game_screen(mut commands: Commands, selected_character: Res<SelectedCharacter>) {
    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.4, 0.8)),
            InGameScreen,
        ))
        .with_children(|parent| {
            // Character name display
            let character_name = match *selected_character {
                SelectedCharacter::Megaman => "Megaman",
                SelectedCharacter::Protoman => "Protoman",
            };
            parent.spawn((
                Text::new(format!("Playing as: {}", character_name)),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
        });
}

/// Handles character selection button clicks
pub fn handle_character_selection(
    interaction_query: Query<
        (&Interaction, &CharacterButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected_character: ResMut<SelectedCharacter>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match button {
                CharacterButton::Megaman => {
                    *selected_character = SelectedCharacter::Megaman;
                    info!("Selected character: Megaman");
                }
                CharacterButton::Protoman => {
                    *selected_character = SelectedCharacter::Protoman;
                    info!("Selected character: Protoman");
                }
            }
            next_state.set(GameState::InGame);
        }
    }
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CharacterSelection), spawn_character_selection_menu)
            .add_systems(
                Update,
                handle_character_selection.run_if(in_state(GameState::CharacterSelection)),
            )
            .add_systems(
                OnExit(GameState::CharacterSelection),
                despawn_screen::<CharacterSelectionMenu>,
            )
            .add_systems(OnEnter(GameState::InGame), spawn_in_game_screen)
            .add_systems(
                OnExit(GameState::InGame),
                despawn_screen::<InGameScreen>,
            );
    }
}
