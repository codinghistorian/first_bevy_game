use bevy::prelude::*;
use crate::components::player::*;
use crate::stages::game_menu::SelectedCharacter;
use crate::systems::config::SMALL_JUMP_CHARGE_RATIO;

/// Spawns the ingame 2D game scene when entering the InGame state
pub fn spawn_player_and_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected_character: Res<SelectedCharacter>,
) {
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
}


/// Handles player movement (left/right) and jumping in the game
pub fn player_movement(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerVelocity, &mut JumpCharge, Option<&mut Dash>), With<Player>>,
) {
    const SPEED: f32 = 200.0; // Pixels per second
    const DASH_SPEED: f32 = 400.0; // Pixels per second
    const DASH_DURATION: f32 = 0.2; // Seconds
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

    for (entity, mut transform, mut velocity, mut jump_charge, dash) in &mut player_query {
        // Horizontal movement
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        if let Some(mut dash) = dash {
            transform.translation.x += dash.direction * DASH_SPEED * time.delta_secs();
            dash.timer -= time.delta_secs();
            if dash.timer <= 0.0 {
                commands.entity(entity).remove::<Dash>();
            }
            return; // No other movement during dash
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

        // Dash
        if keyboard_input.pressed(KeyCode::ArrowDown) && jump_button_just_pressed && is_on_ground {
            let dash_direction = if direction != 0.0 { direction } else { 1.0 };
            commands.entity(entity).insert(Dash {
                timer: DASH_DURATION,
                direction: dash_direction,
            });
            return; // No other movement during dash
        }

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
                if charge_ratio < SMALL_JUMP_CHARGE_RATIO {
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