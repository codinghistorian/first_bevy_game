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

/// Resource to track which character box is currently selected (0 = Megaman, 1 = Protoman)
#[derive(Resource, Default)]
pub struct SelectedCharacterIndex(pub usize);

/// Marker component for the character selection menu UI root
#[derive(Component)]
pub struct CharacterSelectionMenu;

/// Marker component for the ingame screen UI root
#[derive(Component)]
pub struct InGameScreen;

/// Marker component for the player character
#[derive(Component)]
pub struct Player;

/// Marker component for the floor/platform
#[derive(Component)]
pub struct Floor;

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected_character: Res<SelectedCharacter>,
) {
    // Spawn game camera (separate from UI camera)
    commands.spawn(Camera2d);

    // Determine character color based on selection
    let character_color = match *selected_character {
        SelectedCharacter::Megaman => Color::srgb(0.2, 0.4, 0.9), // Blue
        SelectedCharacter::Protoman => Color::srgb(0.9, 0.2, 0.2), // Red
    };

    // Spawn the player character as a rectangle
    // Floor top is at y = -230 (floor center -250 + half-height 20)
    // Character center should be at floor top + character half-height = -230 + 32 = -198
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(32.0, 64.0))), // 32x64 rectangle
        MeshMaterial2d(materials.add(character_color)),
        Transform::from_xyz(0.0, -198.0, 1.0), // Positioned on top of the floor
        Player,
    ));

    // Spawn the floor/platform at the bottom
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(800.0, 40.0))), // Wide floor
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.3, 0.3))), // Gray floor
        Transform::from_xyz(0.0, -250.0, 0.0), // Position at bottom
        Floor,
    ));

    // Background color via clear color (black)
    commands.insert_resource(ClearColor(Color::BLACK));
}

/// Handles player movement (left/right) in the game
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    const SPEED: f32 = 200.0; // Pixels per second

    for mut transform in &mut player_query {
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        if direction != 0.0 {
            transform.translation.x += direction * SPEED * time.delta_secs();
            
            // Optional: Keep player within screen bounds (adjust as needed)
            transform.translation.x = transform.translation.x.clamp(-350.0, 350.0);
        }
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

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCharacterIndex>()
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
            .add_systems(
                Update,
                player_movement.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (despawn_screen::<Player>, despawn_screen::<Floor>),
            );
    }
}
