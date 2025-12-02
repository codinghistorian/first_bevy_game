use crate::stages::game_menu::GameState;
use bevy::text::{Justify, LineBreak, TextLayout};
use bevy::{prelude::*, sprite::Text2d};

/// Marker component for the opening crawl screen
#[derive(Component)]
pub struct OpeningCrawlScreen;

/// Component for crawl text that scrolls upward
#[derive(Component)]
pub struct CrawlText;

/// Component for the text container that scrolls
#[derive(Component)]
pub struct CrawlTextContainer {
    pub scroll_position: f32,
}

/// Component for star sprites in the background
#[derive(Component)]
pub struct Star;

/// Spawns the opening crawl screen with space background and scrolling text
pub fn spawn_opening_crawl(mut commands: Commands) {
    // Set black background
    commands.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)));

    // Spawn a 3D camera with perspective projection for Star Wars effect
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        // Position camera to look down at the tilted text plane
        Transform::from_xyz(0.0, 50.0, 200.0).looking_at(Vec3::new(0.0, -100.0, -200.0), Vec3::Y),
        GlobalTransform::default(),
    ));

    // Spawn stars in the background - create a starfield around the text plane
    for i in 0..300 {
        let seed = i as f32 * 12.9898;
        // Create stars in a large 3D volume around the camera
        let x = (seed.sin() * 10000.0).fract() * 2000.0 - 1000.0;
        let y = (seed.cos() * 10000.0).fract() * 2000.0 - 1000.0;
        let z = ((i as f32 * 7.1234).sin() * 10000.0).fract() * 800.0 - 400.0;

        let size_seed = i as f32 * 7.1234;
        let size = (size_seed.sin() * 10000.0).fract() * 3.0 + 0.5;
        let brightness_seed = i as f32 * 3.4567;
        let brightness = (brightness_seed.sin() * 10000.0).fract() * 0.7 + 0.3;

        commands.spawn((
            Sprite {
                color: Color::srgb(brightness, brightness, brightness),
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_xyz(x, y, z),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Star,
        ));
    }

    // The opening text
    let opening_text = "A long time ago in a bakery far, far away... The Breadman was working in his Bakery when suddenly he was attacked by possessed foods! The Demon Food Lord had contaminated the souls of breads, cakes, pies, and pastries, turning them into vengeful spirits. Cookies crumbled into weapons, cakes rose up to smother him, and donuts rolled like deadly wheels of destruction!";

    // Manual word wrapping into a single multi-line string
    let words: Vec<&str> = opening_text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let max_chars = 40;

    for word in words {
        if current_line.len() + word.len() + 1 > max_chars {
            lines.push(current_line);
            current_line = String::new();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let formatted_text = lines.join("\n\n");

    // Single text plane, tilted back like the Star Wars crawl.
    // Start well below the bottom of the screen so the crawl rises into view.
    let text_transform = Transform::from_translation(Vec3::new(0.0, -320.0, -150.0))
        .with_rotation(Quat::from_rotation_x(-30.0_f32.to_radians()));

    commands.spawn((
        Text2d::new(formatted_text),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.0)), // Gold color
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        CrawlText,
        CrawlTextContainer {
            scroll_position: 0.0,
        },
        OpeningCrawlScreen,
        text_transform,
        Visibility::Visible,
    ));

    // Spawn skip instruction text (UI)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            },
            OpeningCrawlScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Press Enter to Skip"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)), // Grey color
            ));
        });
}

/// Animates the crawl text scrolling upward along the tilted plane
pub fn animate_crawl_text(
    time: Res<Time>,
    mut text_query: Query<(&mut CrawlTextContainer, &mut Transform), With<CrawlText>>,
) {
    const SCROLL_SPEED: f32 = 25.0; // world units per second

    for (mut container, mut transform) in text_query.iter_mut() {
        container.scroll_position += SCROLL_SPEED * time.delta_secs();

        // Move "up" in the text's local space so the perspective and tilt are preserved.
        // This is the key to the Star Wars-style crawl: the whole text plane slides away
        // from the camera along its tilted surface.
        let local_up = transform.rotation * Vec3::Y;
        transform.translation += local_up * SCROLL_SPEED * time.delta_secs();
    }
}

/// Handles input for the opening crawl (skip with any key or auto-transition after duration)
pub fn handle_opening_crawl_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<Option<f32>>,
    time: Res<Time>,
    text_container_query: Query<&CrawlTextContainer>,
) {
    // Auto-transition after 30 seconds
    let auto_transition_time = 30.0;
    
    let elapsed = timer.get_or_insert(0.0);
    *elapsed += time.delta_secs();
    
    // Check if text has scrolled far enough (or skip with any key)
    let text_scrolled = text_container_query.iter().any(|container| container.scroll_position > 1500.0);
    let key_pressed = keyboard_input.just_pressed(KeyCode::Enter)
        || keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Escape);
    let should_transition = key_pressed
        || *elapsed >= auto_transition_time
        || text_scrolled;
    
    if should_transition {
        next_state.set(GameState::CharacterSelection);
    }
}

