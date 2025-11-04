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

/// Component to track player velocity (for jumping and gravity)
#[derive(Component)]
pub struct PlayerVelocity {
    pub y: f32,
    pub jump_type: JumpType,
}

/// Component to track jump charging (hold duration)
#[derive(Component)]
pub struct JumpCharge {
    pub timer: f32,
    pub is_charging: bool,
}

/// Type of jump the player is currently performing
#[derive(Clone, Copy, PartialEq)]
pub enum JumpType {
    None,
    High,
    Small,
}

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
        PlayerVelocity { 
            y: 0.0,
            jump_type: JumpType::None,
        },
        JumpCharge {
            timer: 0.0,
            is_charging: false,
        },
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

/// Handles player movement (left/right) and jumping in the game
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut PlayerVelocity, &mut JumpCharge), With<Player>>,
) {
    const SPEED: f32 = 200.0; // Pixels per second
    const BASE_JUMP_STRENGTH: f32 = 400.0; // Base jump velocity in pixels per second
    const BASE_GRAVITY: f32 = 800.0; // Base gravity acceleration in pixels per second squared
    const GROUND_Y: f32 = -198.0; // Ground level (character center when on floor)
    
    // High jump: 10% higher (1.1x), 10% faster gravity (1.1x)
    const HIGH_JUMP_STRENGTH: f32 = BASE_JUMP_STRENGTH * 1.1; // 440.0
    const HIGH_JUMP_GRAVITY: f32 = BASE_GRAVITY * 1.1; // 880.0
    
    // Small jump: 40% of base jump (0.4x), 20% faster gravity (1.2x)
    const SMALL_JUMP_STRENGTH: f32 = BASE_JUMP_STRENGTH * 0.4; // 160.0
    const SMALL_JUMP_GRAVITY: f32 = BASE_GRAVITY * 1.2; // 960.0
    
    const MAX_CHARGE_TIME: f32 = 0.2; // Maximum charge time for high jump (0.2 seconds)

    for (mut transform, mut velocity, mut jump_charge) in &mut player_query {
        // Horizontal movement
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        if direction != 0.0 {
            transform.translation.x += direction * SPEED * time.delta_secs();
            
            // Keep player within screen bounds
            transform.translation.x = transform.translation.x.clamp(-350.0, 350.0);
        }

        // Check if jump button is pressed (Arrow Up, Space, or X)
        let jump_button_pressed = keyboard_input.pressed(KeyCode::ArrowUp)
            || keyboard_input.pressed(KeyCode::Space)
            || keyboard_input.pressed(KeyCode::KeyX);
        let jump_button_just_pressed = keyboard_input.just_pressed(KeyCode::ArrowUp)
            || keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::KeyX);
        let jump_button_just_released = keyboard_input.just_released(KeyCode::ArrowUp)
            || keyboard_input.just_released(KeyCode::Space)
            || keyboard_input.just_released(KeyCode::KeyX);

        let is_on_ground = transform.translation.y <= GROUND_Y;

        // Start charging jump when button is pressed on ground
        if jump_button_just_pressed && is_on_ground {
            jump_charge.is_charging = true;
            jump_charge.timer = 0.0;
        }

        // Charge jump while button is held
        if jump_charge.is_charging && jump_button_pressed && is_on_ground {
            jump_charge.timer += time.delta_secs();
        }

        // Execute jump when button is released
        if jump_button_just_released && jump_charge.is_charging {
            if is_on_ground {
                // Calculate jump strength based on charge time
                let charge_ratio = (jump_charge.timer / MAX_CHARGE_TIME).clamp(0.0, 1.0);
                
                // Interpolate between small and high jump based on charge time
                if charge_ratio < 0.5 {
                    // Short press = small jump
                    velocity.y = SMALL_JUMP_STRENGTH;
                    velocity.jump_type = JumpType::Small;
                } else {
                    // Long press = high jump
                    velocity.y = HIGH_JUMP_STRENGTH;
                    velocity.jump_type = JumpType::High;
                }
            }
            
            // Reset charge
            jump_charge.is_charging = false;
            jump_charge.timer = 0.0;
        }
        
        // Determine gravity based on current jump type
        let current_gravity = match velocity.jump_type {
            JumpType::High => HIGH_JUMP_GRAVITY,
            JumpType::Small => SMALL_JUMP_GRAVITY,
            JumpType::None => BASE_GRAVITY,
        };

        // Apply gravity only when in the air
        if !is_on_ground {
            velocity.y -= current_gravity * time.delta_secs();
        }

        // Apply vertical velocity
        transform.translation.y += velocity.y * time.delta_secs();

        // Ground collision - stop falling when hitting the ground
        if transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.y = 0.0;
            velocity.jump_type = JumpType::None; // Reset jump type when landing
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
