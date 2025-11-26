use crate::components::boss::{Boss, BossType};
use crate::components::player::{HealthBar, HealthBarBackground, HealthBarMask, Hp, Player};
use crate::config::gameplay::{BOUNDARY_LEFT, BOUNDARY_TOP};
use crate::config::ui::{PLAYER_HP_BAR_MARGIN_LEFT, PLAYER_HP_BAR_RADIUS};
use crate::stages::game_menu::{CurrentStage, DefeatedBoss, GameState, PlayerUpgrades};
use bevy::prelude::*;

/// Spawns the player's HP bar as a circular bar at the top-left (Diablo 2 style - drains from top).
pub fn setup_player_hp_bar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<Entity, With<Player>>,
) {
    let Some(player) = player_query.iter().next() else {
        // Player doesn't exist yet, skip creating HP bar
        return;
    };

    // Calculate position: top-left, with Y at the ceiling (BOUNDARY_TOP)
    let screen_y = BOUNDARY_TOP;
    let screen_x = BOUNDARY_LEFT + PLAYER_HP_BAR_MARGIN_LEFT + PLAYER_HP_BAR_RADIUS;

    // Spawn circular HP bar background (outer circle - black border)
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_HP_BAR_RADIUS))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(screen_x, screen_y, 2.0), // Z=2.0 to be above game elements
        HealthBarBackground,
    ));

    // Spawn circular HP bar fill (inner circle that drains from top)
    // We'll use a rectangle mask approach: the fill circle is clipped from the top based on HP
    let fill_radius = PLAYER_HP_BAR_RADIUS - 4.0; // Slightly smaller for border effect

    // Create the fill circle
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(fill_radius))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0))), // Green
        Transform::from_xyz(screen_x, screen_y, 2.1),              // Slightly above background
        HealthBar { entity: player },
    ));

    // Spawn a rectangular mask above the fill circle to hide the top portion.
    // This achieves a linear "drain from top" visual without distorting the circle.
    let diameter = fill_radius * 2.0;
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(diameter, diameter))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(screen_x, screen_y, 2.2), // Above the fill
        HealthBarMask { entity: player },
    ));
}

/// Compute the area of a circular segment (cap) with a given height.
fn circular_segment_area(height: f32, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }

    let clamped_height = height.clamp(0.0, 2.0 * radius);
    if clamped_height <= f32::EPSILON {
        return 0.0;
    }

    if (clamped_height - 2.0 * radius).abs() <= f32::EPSILON {
        return std::f32::consts::PI * radius * radius;
    }

    let r = radius;
    let h = clamped_height;
    let term = ((r - h) / r).clamp(-1.0, 1.0);
    let theta = term.acos();
    let sqrt_term = (2.0 * r * h - h * h).max(0.0).sqrt();

    r * r * theta - (r - h) * sqrt_term
}

/// Convert a missing area fraction into a mask height so that the visible
/// portion of the HP orb matches the remaining HP percentage.
fn segment_height_for_fraction(fraction: f32, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }

    let frac = fraction.clamp(0.0, 1.0);
    if frac <= f32::EPSILON {
        return 0.0;
    }

    if frac >= 1.0 - f32::EPSILON {
        return 2.0 * radius;
    }

    let target_area = frac * std::f32::consts::PI * radius * radius;
    let mut low = 0.0;
    let mut high = 2.0 * radius;

    for _ in 0..20 {
        let mid = 0.5 * (low + high);
        let area = circular_segment_area(mid, radius);

        if (area - target_area).abs() <= 1e-4 {
            return mid;
        }

        if area < target_area {
            low = mid;
        } else {
            high = mid;
        }
    }

    0.5 * (low + high)
}

/// System to update the health bars based on the entity's HP.
/// Handles both circular HP bars (player - Diablo 2 style) and rectangular HP bars (boss).
pub fn update_health_bars(
    hp_query: Query<&Hp>,
    // Query for circular HP bars (player) - uses Mesh2d with Transform and MeshMaterial2d
    mut circular_health_bar_query: Query<
        (&HealthBar, &mut MeshMaterial2d<ColorMaterial>),
        (With<Mesh2d>, Without<Node>, Without<HealthBarMask>),
    >,
    // Query for circular HP mask rectangles (player), disjoint from the fill
    mut mask_query: Query<(&HealthBarMask, &mut Transform), (Without<HealthBar>,)>,
    // Query for rectangular HP bars (boss) - uses UI Node
    mut rectangular_health_bar_query: Query<
        (&HealthBar, &mut Node),
        (With<Node>, Without<Mesh2d>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Update circular HP bars (player) - keep circle shape, only change color
    for (health_bar, mesh_material) in circular_health_bar_query.iter_mut() {
        if let Ok(hp) = hp_query.get(health_bar.entity) {
            let health_percentage = (hp.current / hp.max).clamp(0.0, 1.0);

            // Change color based on HP (green -> yellow -> red)
            let color = if health_percentage > 0.5 {
                // Green to yellow transition
                let t = (health_percentage - 0.5) * 2.0;
                Color::srgb(1.0 - t, 1.0, 0.0)
            } else {
                // Yellow to red transition
                let t = health_percentage * 2.0;
                Color::srgb(1.0, t, 0.0)
            };

            // Update the material color
            if let Some(material) = materials.get_mut(&mesh_material.0) {
                material.color = color;
            }
        }
    }

    // Update the rectangular mask to linearly hide the top portion of the circle
    for (mask, mut transform) in mask_query.iter_mut() {
        if let Ok(hp) = hp_query.get(mask.entity) {
            let health_percentage = (hp.current / hp.max).clamp(0.0, 1.0);
            let missing_fraction = (1.0 - health_percentage).clamp(0.0, 1.0);

            let fill_radius = PLAYER_HP_BAR_RADIUS - 4.0;
            let diameter = fill_radius * 2.0;
            let base_y = BOUNDARY_TOP;

            // Convert missing health into a circular segment height so that the
            // visible area of the orb matches the remaining HP percentage.
            let mask_height = segment_height_for_fraction(missing_fraction, fill_radius);
            let y_scale = if diameter > 0.0 {
                (mask_height / diameter).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let scaled_half_height = mask_height * 0.5;

            // Position the mask so its top edge aligns with the top of the circle,
            // and it grows downward as HP is lost.
            let mask_center_y = base_y + fill_radius - scaled_half_height;
            transform.scale = Vec3::new(1.0, y_scale, 1.0);
            transform.translation.y = mask_center_y;
        }
    }

    // Update rectangular HP bars (boss) - existing UI-based system
    for (health_bar, mut node) in rectangular_health_bar_query.iter_mut() {
        if let Ok(hp) = hp_query.get(health_bar.entity) {
            let health_percentage = (hp.current / hp.max) * 100.0;
            node.width = Val::Percent(health_percentage);
        }
    }
}

/// System to handle health regeneration (currently disabled - player doesn't regenerate)
/// This can be enabled later if you want health regeneration mechanics
pub fn change_health(_time: Res<Time>, _player_query: Query<&mut Hp, With<Player>>) {
    // Health regeneration disabled - player HP stays at current value
    // Uncomment below to enable regeneration:
    // let mut player_hp = player_query.single_mut().unwrap();
    // player_hp.current = (player_hp.current + 5.0 * time.delta_secs()).min(player_hp.max);
}

/// System to persist player HP to PlayerUpgrades resource
pub fn persist_player_hp(
    player_query: Query<&Hp, With<Player>>,
    mut player_upgrades: ResMut<PlayerUpgrades>,
) {
    if let Some(player_hp) = player_query.iter().next() {
        // Update the persisted current HP
        player_upgrades.current_hp = player_hp.current;
    }
}

/// System to check for win/lose conditions
pub fn check_game_outcome(
    player_query: Query<&Hp, With<Player>>,
    boss_query: Query<(&Hp, &BossType), With<Boss>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut defeated_boss: ResMut<DefeatedBoss>,
    _current_stage: ResMut<CurrentStage>,
) {
    // Check if player is dead (lose condition)
    if let Some(player_hp) = player_query.iter().next() {
        if player_hp.current <= 0.0 {
            next_state.set(GameState::GameOver);
            return;
        }
    }

    // Check if boss is dead (win condition)
    if let Some((boss_hp, boss_type)) = boss_query.iter().next() {
        if boss_hp.current <= 0.0 {
            // Store which boss was defeated
            defeated_boss.boss_type = Some(*boss_type);

            // Always transition to GameWin screen
            // The handle_stage_progression system will check if we should continue to next stage
            next_state.set(GameState::GameWin);
        }
    }
}

