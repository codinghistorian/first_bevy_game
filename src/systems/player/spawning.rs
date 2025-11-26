use crate::components::player::{
    AnimationIndices, AnimationState, AnimationTimer, ChargeShot, Floor, Hp, JumpCharge, JumpType,
    Player, PlayerAnimationConfig, PlayerVelocity, Shooting,
};
use crate::stages::game_menu::{PlayerUpgrades, SelectedCharacter};
use bevy::prelude::*;

/// Spawns the ingame 2D game scene when entering the InGame state
pub fn spawn_player_and_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    selected_character: Res<SelectedCharacter>,
    player_upgrades: Option<Res<PlayerUpgrades>>,
) {
    // Determine character color based on selection
    let character_color = match *selected_character {
        SelectedCharacter::Breadman => Color::srgb(0.2, 0.4, 0.9), // Blue
        SelectedCharacter::Cheeseman => Color::srgb(0.9, 0.2, 0.2), // Red
    };

    // Calculate HP with upgrades
    let base_max_hp = 100.0;
    let max_hp_bonus = player_upgrades
        .as_ref()
        .map(|u| u.max_hp_bonus)
        .unwrap_or(0.0);
    let max_hp = base_max_hp + max_hp_bonus;

    // Use preserved current HP if available, otherwise start with full HP
    let current_hp = player_upgrades
        .as_ref()
        .map(|u| u.current_hp.min(max_hp)) // Ensure current HP doesn't exceed new max HP
        .unwrap_or(max_hp);

    // Common player components
    let player_components = (
        Player,
        Hp {
            current: current_hp, // Start with preserved HP or full HP
            max: max_hp,
        },
        PlayerVelocity {
            y: 0.0,
            jump_type: JumpType::None,
            facing_direction: Vec2::new(1.0, 0.0),
            wall_slide: None,
            can_wall_jump: false,
            wall_jump_velocity: 0.0,
            has_wall_jumped: false,
            wall_detach_timer: 0.0,
            last_wall_side: None,
        },
        JumpCharge {
            timer: 0.0,
            is_charging: false,
        },
        Shooting {
            timer: 0.0,
            is_charging: false,
        },
        ChargeShot {
            timer: 0.0,
            is_charging: false,
        },
    );

    // Spawn the player character
    // Floor top is at y = -230 (floor center -250 + half-height 20)
    // Character center should be at floor top + character half-height = -230 + 32 = -198
    let start_transform = Transform::from_xyz(0.0, -198.0, 1.0);

    if matches!(*selected_character, SelectedCharacter::Breadman) {
        // Use sprite sheet for Breadman
        let texture_handle = asset_server.load("images/breadman/megaman_sheet.png");
        
        // Create a custom layout since the sprite sheet is not a uniform grid
        // The image is 349x381, so we use that as the total size
        let mut layout = TextureAtlasLayout::new_empty(UVec2::new(349, 381));

        // --- DEFINE SPRITE COORDINATES HERE ---
        // TODO: You must measure your sprites in the PNG and put the exact coordinates here.
        // Format: URect::new(left_x, top_y, right_x, bottom_y)
        
        // 1. IDLE FRAME (Standing still)
        let idle_idx = layout.add_texture(URect::new(10, 10, 42, 42)); // Placeholder coords

        // 2. RUN FRAMES (Running animation)
        // Add each frame of the run animation in order
        let run_start = layout.add_texture(URect::new(50, 10, 82, 42)); // Run frame 1
        let _         = layout.add_texture(URect::new(90, 10, 122, 42)); // Run frame 2
        let run_end   = layout.add_texture(URect::new(130, 10, 162, 42)); // Run frame 3

        // 3. JUMP FRAME
        let jump_idx = layout.add_texture(URect::new(10, 50, 42, 82)); // Jump frame

        let layout_handle = texture_atlas_layouts.add(layout);

        // Define animation configuration using the indices we just created
        let animation_config = PlayerAnimationConfig {
            idle: AnimationIndices { first: idle_idx, last: idle_idx },
            run: AnimationIndices { first: run_start, last: run_end },
            jump: AnimationIndices { first: jump_idx, last: jump_idx },
        };

        commands.spawn((
            Sprite {
                image: texture_handle,
                texture_atlas: Some(TextureAtlas {
                    layout: layout_handle,
                    index: animation_config.idle.first,
                }),
                custom_size: Some(Vec2::new(64.0, 64.0)), // Scale up to match hit box size approx
                ..default()
            },
            start_transform,
            player_components,
            animation_config,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            AnimationState::Idle,
        ));
    } else {
        // Use rectangle for other characters (Cheeseman)
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(32.0, 64.0))), // 32x64 rectangle
            MeshMaterial2d(materials.add(character_color)),
            start_transform,
            player_components,
        ));
    }

    // Spawn the floor/platform at the bottom
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(800.0, 40.0))), // Wide floor
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.3, 0.3))), // Gray floor
        Transform::from_xyz(0.0, -250.0, 0.0),           // Position at bottom
        Floor,
    ));
}

