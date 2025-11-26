use crate::config::gameplay::{
    BACKGROUND_PADDING, BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP,
};
use crate::stages::game_menu::CurrentStage;
use bevy::{prelude::*, sprite::Anchor};

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

/// Loads background images for each stage dynamically by iterating through available images
pub fn load_background_images(
    mut background_images: ResMut<BackgroundImages>,
    asset_server: Res<AssetServer>,
) {
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
    info!(
        "Attempted to load up to {} background images for stage 1",
        MAX_IMAGES
    );
    info!(
        "Loaded {} background image handles for stage 1",
        background_images.stage_1.len()
    );
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
            info!(
                "First background image load state: {:?}, handle id: {:?}",
                load_state,
                first_handle.id()
            );

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
            warn!(
                "No background images available for stage {}",
                current_stage.0
            );
            commands.insert_resource(ClearColor(Color::BLACK));
        }
    } else {
        info!(
            "No background images configured for stage {}",
            current_stage.0
        );
        commands.insert_resource(ClearColor(Color::BLACK));
    }
}

