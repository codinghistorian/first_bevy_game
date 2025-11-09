# Bevy 0.17.2 Background Image Rendering Issue

## Problem
I'm trying to render background images in a Bevy 0.17.2 game, but the images are not appearing on screen. The images should cycle through 4 JPG files for stage 1, but nothing is rendered.

## Environment
- **Bevy Version**: 0.17.2
- **Rust Edition**: 2024
- **OS**: macOS

## Project Structure
```
first_bevy_game/
├── images/
│   └── backgrounds/
│       └── stage_1/
│           ├── stage_1_1.jpg
│           ├── stage_1_2.jpg
│           ├── stage_1_3.jpg
│           └── stage_1_4.jpg
├── src/
│   ├── main.rs
│   ├── stages/
│   │   └── game_menu.rs (contains background loading/spawning code)
│   └── systems/
│       └── config.rs (contains boundary constants)
└── Cargo.toml
```

## Asset Configuration
In `main.rs`, the AssetPlugin is configured to use the project root as the asset directory:
```rust
.add_plugins(DefaultPlugins.set(AssetPlugin {
    file_path: ".".into(),
    ..default()
}))
```

## Current Implementation

### Background Loading (runs on Startup)
```rust
pub fn load_background_images(mut background_images: ResMut<BackgroundImages>, asset_server: Res<AssetServer>) {
    info!("Loading background images for stage 1...");
    let handles = vec![
        asset_server.load("images/backgrounds/stage_1/stage_1_1.jpg"),
        asset_server.load("images/backgrounds/stage_1/stage_1_2.jpg"),
        asset_server.load("images/backgrounds/stage_1/stage_1_3.jpg"),
        asset_server.load("images/backgrounds/stage_1/stage_1_4.jpg"),
    ];
    background_images.stage_1 = handles;
    // ... logging code
}
```

### Background Spawning (runs on OnEnter(GameState::InGame))
```rust
pub fn spawn_in_game_screen(
    mut commands: Commands,
    background_images: Res<BackgroundImages>,
    mut current_stage: ResMut<CurrentStage>,
    asset_server: Res<AssetServer>,
) {
    // Spawn game camera
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Ensure stage is set to 1
    if current_stage.0 == 0 {
        current_stage.0 = 1;
    }

    let stage_number = current_stage.0;

    if let Some(image_handles) = background_images.get_stage_images(stage_number) {
        if !image_handles.is_empty() {
            let first_handle = &image_handles[0];
            let load_state = asset_server.load_state(first_handle);
            
            // Calculate background size to fit game boundaries
            let bg_width = BOUNDARY_RIGHT - BOUNDARY_LEFT;  // 700.0
            let bg_height = BOUNDARY_TOP - BOUNDARY_BOTTOM; // 398.0
            let bg_center_x = (BOUNDARY_LEFT + BOUNDARY_RIGHT) / 2.0;  // 0.0
            let bg_center_y = (BOUNDARY_BOTTOM + BOUNDARY_TOP) / 2.0;  // 1.0
            
            // Spawn background sprite
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
                BackgroundImage, // Custom component marker
            ));
        }
    }
}
```

### Game Boundaries (from config.rs)
```rust
pub const BOUNDARY_LEFT: f32 = -350.0;
pub const BOUNDARY_RIGHT: f32 = 350.0;
pub const BOUNDARY_TOP: f32 = 200.0;
pub const BOUNDARY_BOTTOM: f32 = -198.0;
```

## What I've Tried
1. ✅ Using `Sprite` component with `image` field and `custom_size`
2. ✅ Adding all visibility components (`Visibility`, `InheritedVisibility`, `ViewVisibility`)
3. ✅ Adding `Transform` and `GlobalTransform`
4. ✅ Using `Anchor::CENTER` for positioning
5. ✅ Setting z-index to -10.0 to ensure it's behind other entities
6. ✅ Configuring AssetPlugin to use project root (`.`)
7. ✅ Adding debug logging to check asset load states
8. ✅ Ensuring camera is spawned before background
9. ✅ Calculating size based on game boundaries

## What's NOT Working
- The background images do not render/display on screen
- No compilation errors
- Assets appear to be loading (based on logs)
- Other game entities (player, boss, floor) render correctly

## Questions
1. Is the sprite spawning code correct for Bevy 0.17.2?
2. Are all required components present for sprite rendering?
3. Is the asset path configuration correct?
4. Should I be using a different approach (e.g., `SpriteBundle`, `Image2d`, etc.)?
5. Could there be a camera/viewport issue preventing the background from being visible?
6. Should I wait for assets to fully load before spawning the sprite?

## Additional Context
- The game uses a state machine with `GameState::InGame` for gameplay
- There's a separate UI camera (order: 1) and game camera (order: 0)
- The background should animate by cycling through the 4 images every 0.5 seconds
- Other sprites/entities in the game render fine, so the camera setup seems correct

Please provide a working solution for rendering background images in Bevy 0.17.2, or identify what's wrong with the current implementation.

