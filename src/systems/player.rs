use crate::components::boss::*;
use crate::components::player::{ChargeEffect, ChargeShot, *};
use crate::stages::game_menu::PlayerUpgrades;
use crate::stages::game_menu::{DefeatedBoss, GameState, SelectedCharacter};
use crate::systems::config::{
    BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, CHARGE_SHOT_COOLDOWN,
    CHARGE_SHOT_DAMAGE_MULTIPLIER, CHARGE_SHOT_MAX_TIME, CHARGE_SHOT_MIN_TIME,
    INVINCIBILITY_DURATION, KNOCKBACK_DECAY_RATE, KNOCKBACK_DURATION, KNOCKBACK_FORCE,
    KNOCKBACK_MOVEMENT_REDUCTION, NORMAL_SHOT_COOLDOWN, PLAYER_HP_BAR_MARGIN_LEFT,
    PLAYER_HP_BAR_RADIUS, PLAYER_PROJECTILE_DAMAGE, SMALL_JUMP_CHARGE_RATIO,
};
use bevy::prelude::*;

/// Spawns the ingame 2D game scene when entering the InGame state
pub fn spawn_player_and_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    // Spawn the player character as a rectangle
    // Floor top is at y = -230 (floor center -250 + half-height 20)
    // Character center should be at floor top + character half-height = -230 + 32 = -198
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(32.0, 64.0))), // 32x64 rectangle
        MeshMaterial2d(materials.add(character_color)),
        Transform::from_xyz(0.0, -198.0, 1.0), // Positioned on top of the floor
        Player,
        Hp {
            current: current_hp, // Start with preserved HP or full HP
            max: max_hp,
        },
        PlayerVelocity {
            y: 0.0,
            jump_type: JumpType::None,
            facing_direction: Vec2::new(1.0, 0.0),
        },
        JumpCharge {
            timer: 0.0,
            is_charging: false,
        },
        Shooting { timer: 0.0 },
        ChargeShot {
            timer: 0.0,
            is_charging: false,
        },
    ));

    // Spawn the floor/platform at the bottom
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(800.0, 40.0))), // Wide floor
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.3, 0.3))), // Gray floor
        Transform::from_xyz(0.0, -250.0, 0.0),           // Position at bottom
        Floor,
    ));
}

/// Spawns the boss on the right side of the game field
pub fn spawn_boss(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    boss_registry: Option<Res<BossRegistry>>,
    pattern_registry: Option<Res<crate::systems::boss::BossPatternRegistry>>,
    current_stage: Option<Res<crate::stages::game_menu::CurrentStage>>,
) {
    use crate::systems::boss::{convert_attack_pattern, convert_movement_pattern};

    // Get boss data from registry or use default
    let mut boss_data = boss_registry
        .as_ref()
        .and_then(|registry| registry.get_boss_data(BossType::Default))
        .cloned()
        .unwrap_or_else(|| BossData::default());

    // Try to load pattern from JSON based on stage number
    if let (Some(registry), Some(stage)) = (pattern_registry.as_ref(), current_stage.as_ref()) {
        let stage_num = stage.0;
        let pattern_name = format!("stage_{}", stage_num);

        if let Some(pattern_config) = registry.get_pattern(&pattern_name) {
            // Convert JSON patterns to internal patterns
            boss_data.attack_pattern = convert_attack_pattern(&pattern_config.attack);
            boss_data.movement_pattern = convert_movement_pattern(&pattern_config.movement);
        }
    }

    // Spawn the boss character on the right side
    // Position at x = 300 (right side), same y as player (-198)
    let _boss_entity = commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(boss_data.size.x, boss_data.size.y))),
        MeshMaterial2d(materials.add(boss_data.color)),
        Transform::from_xyz(300.0, -198.0, 1.0), // Positioned on the right side, on top of the floor
        Boss,
        boss_data.boss_type,
        boss_data.clone(),
        Hp {
            current: 200.0,
            max: 200.0,
        },
        BossAttackState::default(),
        BossMovementState::default(),
    ));

    // TODO: Add sprite rendering when sprite is available
    // In Bevy 0.17, you would use Sprite2d or Image2d depending on your setup
    // For now, we use the colored rectangle as fallback
    // if let Some(sprite_handle) = boss_data.sprite {
    //     // Add sprite component here when ready
    // }
}

/// Handles player movement (left/right) and jumping in the game
pub fn player_movement(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &mut PlayerVelocity,
            &mut JumpCharge,
            Option<&mut Dash>,
            Option<&Knockback>,
        ),
        With<Player>,
    >,
) {
    const SPEED: f32 = 200.0; // Pixels per second
    const DASH_SPEED: f32 = 400.0; // Pixels per second
    const DASH_DURATION: f32 = 0.2; // Seconds
    const BASE_JUMP_STRENGTH: f32 = 400.0; // Base jump velocity in pixels per second
    const BASE_GRAVITY: f32 = 800.0; // Base gravity acceleration in pixels per second squared
    const GROUND_Y: f32 = -198.0; // Ground level (character center when on floor)

    // High jump: 10% higher (1.1x), 10% faster gravity (1.1x)
    const HIGH_JUMP_STRENGTH: f32 = 620.0; // 440.0
    const HIGH_JUMP_GRAVITY: f32 = 1200.0; // 880.0

    // Small jump: 40% of base jump (0.4x), 20% faster gravity (1.2x)
    const SMALL_JUMP_STRENGTH: f32 = 350.5; // 160.0
    const SMALL_JUMP_GRAVITY: f32 = BASE_GRAVITY * 1.2; // 960.0

    const MAX_CHARGE_TIME: f32 = 0.2; // Maximum charge time for high jump (0.2 seconds)

    for (entity, mut transform, mut velocity, mut jump_charge, dash, knockback) in &mut player_query
    {
        // Movement
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        // We don't handle ArrowDown for movement, only for dash
        // if keyboard_input.pressed(KeyCode::ArrowDown) {
        //     direction.y -= 1.0;
        // }

        if direction != Vec2::ZERO {
            velocity.facing_direction = direction.normalize();
        }

        if let Some(mut dash) = dash {
            transform.translation.x += dash.direction * DASH_SPEED * time.delta_secs();
            dash.timer -= time.delta_secs();
            if dash.timer <= 0.0 {
                commands.entity(entity).remove::<Dash>();
            }
            return; // No other movement during dash
        }

        // Apply movement, but reduce it if knockback is active
        let movement_speed = if knockback.is_some() {
            SPEED * KNOCKBACK_MOVEMENT_REDUCTION // Reduce movement speed during knockback
        } else {
            SPEED
        };
        transform.translation.x += direction.x * movement_speed * time.delta_secs();
        // Keep player within boundaries
        transform.translation.x = transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
        transform.translation.y = transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);

        // Check if jump button is pressed (Space, or X)
        let jump_button_pressed =
            keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::KeyX);
        let jump_button_just_pressed = keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::KeyX);
        let jump_button_just_released = keyboard_input.just_released(KeyCode::Space)
            || keyboard_input.just_released(KeyCode::KeyX);

        let is_on_ground = transform.translation.y <= GROUND_Y;

        // Dash
        if keyboard_input.pressed(KeyCode::ArrowDown) && jump_button_just_pressed && is_on_ground {
            commands.entity(entity).insert(Dash {
                timer: DASH_DURATION,
                direction: velocity.facing_direction.x,
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

pub fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<
        (&Transform, &PlayerVelocity, &mut Shooting, &mut ChargeShot),
        With<Player>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected_character: Res<SelectedCharacter>,
) {
    let is_breadman = matches!(*selected_character, SelectedCharacter::Breadman);

    for (player_transform, player_velocity, mut shooting, mut charge_shot) in &mut player_query {
        shooting.timer -= time.delta_secs();

        let shoot_button_pressed = keyboard_input.pressed(KeyCode::KeyC);
        let shoot_button_just_pressed = keyboard_input.just_pressed(KeyCode::KeyC);
        let shoot_button_just_released = keyboard_input.just_released(KeyCode::KeyC);

        // Helper function to determine shooting direction
        let get_shoot_direction = || -> Option<Vec2> {
            let shoot_direction;
            // Prioritize vertical over horizontal if both are pressed
            if player_velocity.facing_direction.y > 0.0 {
                // Facing up
                shoot_direction = Vec2::Y;
            } else if player_velocity.facing_direction.x.abs() > 0.0 {
                // Facing left or right
                shoot_direction = Vec2::X * player_velocity.facing_direction.x.signum();
            } else {
                // Default to right if no clear direction (e.g., standing still)
                shoot_direction = Vec2::X;
            }

            // Prevent shooting downwards
            if shoot_direction.y < 0.0 {
                return None;
            }
            Some(shoot_direction)
        };

        // Helper function to spawn a projectile
        let mut spawn_projectile = |direction: Vec2, charge_level: f32, is_charged: bool| {
            let projectile_transform = Transform::from_xyz(
                player_transform.translation.x,
                player_transform.translation.y,
                0.0,
            );

            // Determine projectile size and color based on charge level
            let (size, color) = if is_charged {
                // Charged shot: larger and brighter (yellow/orange)
                let size_multiplier = 1.0 + (charge_level * 1.5); // 1.0x to 2.5x size
                let size = 10.0 * size_multiplier;
                // Color transitions from yellow (low charge) to bright orange/red (full charge)
                let r = 1.0;
                let g = 1.0 - (charge_level * 0.3); // 1.0 to 0.7
                let b = charge_level * 0.2; // 0.0 to 0.2
                (size, Color::srgb(r, g, b))
            } else {
                // Normal shot: small red
                (10.0, Color::srgb(1.0, 0.0, 0.0))
            };

            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(size, size))),
                MeshMaterial2d(materials.add(color)),
                projectile_transform,
                Projectile {
                    direction,
                    charge_level,
                },
            ));
        };

        if is_breadman {
            // Breadman: Charge shot mechanics
            // Start charging when button is pressed
            if shoot_button_just_pressed && shooting.timer <= 0.0 {
                charge_shot.is_charging = true;
                charge_shot.timer = 0.0;
            }

            // Charge while button is held
            if charge_shot.is_charging && shoot_button_pressed {
                charge_shot.timer += time.delta_secs();
                charge_shot.timer = charge_shot.timer.min(CHARGE_SHOT_MAX_TIME);
            }

            // Fire when button is released
            if shoot_button_just_released && charge_shot.is_charging {
                if let Some(shoot_direction) = get_shoot_direction() {
                    let charge_level = (charge_shot.timer / CHARGE_SHOT_MAX_TIME).clamp(0.0, 1.0);
                    let is_charged_shot = charge_shot.timer >= CHARGE_SHOT_MIN_TIME;

                    spawn_projectile(shoot_direction, charge_level, is_charged_shot);

                    // Set cooldown based on shot type
                    shooting.timer = if is_charged_shot {
                        CHARGE_SHOT_COOLDOWN
                    } else {
                        NORMAL_SHOT_COOLDOWN
                    };
                }

                // Reset charge
                charge_shot.is_charging = false;
                charge_shot.timer = 0.0;
            }
        } else {
            // Cheeseman: Normal shots only (no charge)
            // Fire immediately when button is pressed
            if shoot_button_just_pressed && shooting.timer <= 0.0 {
                if let Some(shoot_direction) = get_shoot_direction() {
                    spawn_projectile(shoot_direction, 0.0, false);
                    shooting.timer = NORMAL_SHOT_COOLDOWN;
                }
            }

            // Reset any charge state (in case it was set somehow)
            charge_shot.is_charging = false;
            charge_shot.timer = 0.0;
        }
    }
}

/// System to manage charge effect visual (spawn/despawn based on charging state)
pub fn manage_charge_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<(Entity, &Transform, &ChargeShot), With<Player>>,
    charge_effect_query: Query<(Entity, &ChargeEffect)>,
    selected_character: Res<SelectedCharacter>,
) {
    let is_breadman = matches!(*selected_character, SelectedCharacter::Breadman);

    if !is_breadman {
        // Despawn any charge effects if not Breadman
        for (effect_entity, _) in &charge_effect_query {
            commands.entity(effect_entity).despawn();
        }
        return;
    }

    // Check if player is charging and doesn't have an effect yet
    for (player_entity, player_transform, charge_shot) in &player_query {
        if charge_shot.is_charging {
            // Spawn charge effect if not already present
            let has_effect = charge_effect_query
                .iter()
                .any(|(_, effect)| effect.player_entity == player_entity);

            if !has_effect {
                // Spawn a pulsing circle around the player
                commands.spawn((
                    Mesh2d(meshes.add(Circle::new(40.0))),
                    MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 0.0, 0.3))), // Yellow, semi-transparent
                    Transform::from_translation(player_transform.translation),
                    ChargeEffect {
                        player_entity,
                    },
                ));
            }
        }
    }

    // Despawn charge effects for players that stopped charging
    for (effect_entity, charge_effect) in &charge_effect_query {
        if let Ok((_, _, charge_shot)) = player_query.get(charge_effect.player_entity) {
            if !charge_shot.is_charging {
                commands.entity(effect_entity).despawn();
            }
        } else {
            // Player doesn't exist, despawn effect
            commands.entity(effect_entity).despawn();
        }
    }
}

/// System to animate charge effect (pulsing, color changes based on charge level)
pub fn animate_charge_effect(
    time: Res<Time>,
    player_query: Query<(&Transform, &ChargeShot), With<Player>>,
    mut charge_effect_query: Query<(&ChargeEffect, &mut Transform, &mut MeshMaterial2d<ColorMaterial>), Without<Player>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (charge_effect, mut effect_transform, mesh_material) in &mut charge_effect_query {
        if let Ok((player_transform, charge_shot)) = player_query.get(charge_effect.player_entity) {
            if charge_shot.is_charging {
                // Update position to follow player
                effect_transform.translation = player_transform.translation;

                // Calculate charge level (0.0 to 1.0)
                let charge_level = (charge_shot.timer / CHARGE_SHOT_MAX_TIME).clamp(0.0, 1.0);

                // Pulsing animation: base size + charge-based size + sine wave pulse
                let base_size = 40.0;
                let charge_size = charge_level * 20.0; // Grows up to 20px more when fully charged
                let pulse = (time.elapsed_secs() * 8.0).sin() * 5.0; // Fast pulsing (8 Hz, ±5px)
                let current_size = base_size + charge_size + pulse;

                // Update mesh size (we'll need to recreate the mesh, but for now update scale)
                effect_transform.scale = Vec3::splat(current_size / base_size);

                // Color transitions: yellow -> orange -> red as charge increases
                let r = 1.0;
                let g = 1.0 - (charge_level * 0.5); // 1.0 to 0.5
                let b = charge_level * 0.3; // 0.0 to 0.3
                let alpha = 0.3 + (charge_level * 0.4); // 0.3 to 0.7 (more opaque when charged)

                // Update material color
                if let Some(material) = materials.get_mut(&mesh_material.0) {
                    material.color = Color::srgba(r, g, b, alpha);
                }
            }
        }
    }
}

pub fn projectile_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &Projectile)>,
) {
    const PROJECTILE_SPEED: f32 = 500.0; // Pixels per second

    for (entity, mut transform, projectile) in &mut projectile_query {
        transform.translation.x += projectile.direction.x * PROJECTILE_SPEED * time.delta_secs();
        transform.translation.y += projectile.direction.y * PROJECTILE_SPEED * time.delta_secs();

        // Despawn projectile after it goes outside boundaries
        if transform.translation.x < BOUNDARY_LEFT
            || transform.translation.x > BOUNDARY_RIGHT
            || transform.translation.y < BOUNDARY_BOTTOM
            || transform.translation.y > BOUNDARY_TOP
        {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns the player's HP bar as a circular bar at the top-left (Diablo 2 style - drains from top).
pub fn setup_player_hp_bar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<Entity, With<Player>>,
) {
    let Ok(player) = player_query.single() else {
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
    mut rectangular_health_bar_query: Query<(&HealthBar, &mut Node), (With<Node>, Without<Mesh2d>)>,
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
            node.width = percent(health_percentage);
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

/// Helper function to check AABB (Axis-Aligned Bounding Box) collision
pub fn check_aabb_collision(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    let half_size1 = size1 * 0.5;
    let half_size2 = size2 * 0.5;

    pos1.x - half_size1.x < pos2.x + half_size2.x
        && pos1.x + half_size1.x > pos2.x - half_size2.x
        && pos1.y - half_size1.y < pos2.y + half_size2.y
        && pos1.y + half_size1.y > pos2.y - half_size2.y
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

/// Calculate improved knockback direction based on collision angle
/// This makes knockback feel more dynamic and appropriate for different collision sides
fn calculate_knockback_direction(
    direction_to_player: Vec2,
    _player_pos: Vec3,
    _boss_pos: Vec3,
) -> Vec2 {
    use crate::systems::config::{
        KNOCKBACK_SIDE_VERTICAL_COMPONENT, KNOCKBACK_TOP_HORIZONTAL_COMPONENT,
        KNOCKBACK_TOP_VERTICAL_COMPONENT,
    };

    if direction_to_player.length() < 0.001 {
        // If positions are exactly the same, push to the left
        return Vec2::new(-1.0, 0.0);
    }

    let normalized = direction_to_player.normalize();
    let dx = direction_to_player.x.abs();
    let dy = direction_to_player.y.abs();

    // Determine which side of the boss the player is hitting
    // If vertical distance is greater, it's a top/bottom collision
    // If horizontal distance is greater, it's a left/right collision
    if dy > dx {
        // Top or bottom collision
        if normalized.y > 0.0 {
            // Player is above boss (hitting from top)
            // Push upward and to the side for more dynamic feel
            let horizontal_dir = if normalized.x > 0.0 { 1.0 } else { -1.0 };
            Vec2::new(
                horizontal_dir * KNOCKBACK_TOP_HORIZONTAL_COMPONENT,
                KNOCKBACK_TOP_VERTICAL_COMPONENT,
            )
            .normalize()
        } else {
            // Player is below boss (hitting from bottom)
            // Push downward and to the side
            let horizontal_dir = if normalized.x > 0.0 { 1.0 } else { -1.0 };
            Vec2::new(
                horizontal_dir * KNOCKBACK_TOP_HORIZONTAL_COMPONENT,
                -KNOCKBACK_TOP_VERTICAL_COMPONENT,
            )
            .normalize()
        }
    } else {
        // Left or right collision (side collision)
        // Push horizontally away with slight upward component for more dynamic feel
        let horizontal_dir = if normalized.x > 0.0 { 1.0 } else { -1.0 };
        Vec2::new(horizontal_dir, KNOCKBACK_SIDE_VERTICAL_COMPONENT).normalize()
    }
}

/// System to handle player-boss collision (player takes damage)
pub fn player_boss_collision(
    time: Res<Time>,
    mut player_query: Query<
        (Entity, &Transform, &mut Hp, Option<&mut Invincibility>),
        With<Player>,
    >,
    boss_query: Query<&Transform, With<Boss>>,
    mut commands: Commands,
    player_upgrades: Option<Res<PlayerUpgrades>>,
) {
    const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 64.0);
    const BOSS_SIZE: Vec2 = Vec2::new(32.0, 64.0);

    // Apply defense multiplier to damage
    let defense_multiplier = player_upgrades
        .as_ref()
        .map(|u| u.defense_multiplier)
        .unwrap_or(1.0);
    let DAMAGE = crate::systems::config::BOSS_COLLISION_DAMAGE * defense_multiplier;

    for (player_entity, player_transform, mut player_hp, invincibility) in &mut player_query {
        // Check if player is invincible
        let is_invincible = if let Some(mut inv) = invincibility {
            inv.timer -= time.delta_secs();
            if inv.timer > 0.0 {
                true
            } else {
                commands.entity(player_entity).remove::<Invincibility>();
                false
            }
        } else {
            false
        };

        if is_invincible {
            continue;
        }

        // Check collision with boss
        for boss_transform in &boss_query {
            if check_aabb_collision(
                player_transform.translation,
                PLAYER_SIZE,
                boss_transform.translation,
                BOSS_SIZE,
            ) {
                // Calculate knockback direction based on collision side
                let direction_to_player =
                    (player_transform.translation - boss_transform.translation).truncate();
                let knockback_direction = calculate_knockback_direction(
                    direction_to_player,
                    player_transform.translation,
                    boss_transform.translation,
                );

                // Player takes damage
                player_hp.current = (player_hp.current - DAMAGE).max(0.0);

                // Add invincibility frames
                commands.entity(player_entity).insert(Invincibility {
                    timer: INVINCIBILITY_DURATION,
                });

                // Add knockback effect
                commands.entity(player_entity).insert(Knockback {
                    velocity: knockback_direction * KNOCKBACK_FORCE,
                    timer: KNOCKBACK_DURATION,
                });

                // Only process one collision per frame
                break;
            }
        }
    }
}

/// System to apply knockback effect to player
pub fn apply_knockback(
    time: Res<Time>,
    mut player_query: Query<(Entity, &mut Transform, &mut Knockback), With<Player>>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut knockback) in &mut player_query {
        // Apply knockback velocity
        transform.translation.x += knockback.velocity.x * time.delta_secs();
        transform.translation.y += knockback.velocity.y * time.delta_secs();

        // Keep player within boundaries even during knockback
        transform.translation.x = transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
        transform.translation.y = transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);

        // Decay knockback over time
        knockback.velocity *= KNOCKBACK_DECAY_RATE; // Reduce velocity each frame
        knockback.timer -= time.delta_secs();

        // Remove knockback when timer expires
        if knockback.timer <= 0.0 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

/// System to apply knockback effect to boss
pub fn apply_boss_knockback(
    time: Res<Time>,
    mut boss_query: Query<(Entity, &mut Transform, &mut Knockback), With<Boss>>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut knockback) in &mut boss_query {
        // Apply knockback velocity
        transform.translation.x += knockback.velocity.x * time.delta_secs();
        transform.translation.y += knockback.velocity.y * time.delta_secs();

        // Keep boss within boundaries even during knockback
        transform.translation.x = transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
        transform.translation.y = transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);

        // Decay knockback over time
        knockback.velocity *= KNOCKBACK_DECAY_RATE; // Reduce velocity each frame
        knockback.timer -= time.delta_secs();

        // Remove knockback when timer expires
        if knockback.timer <= 0.0 {
            commands.entity(entity).remove::<Knockback>();
        }
    }
}

/// System to handle projectile-boss collision (boss takes damage, projectile despawns)
pub fn projectile_boss_collision(
    mut commands: Commands,
    projectile_query: Query<
        (Entity, &Transform, &Projectile),
        (
            With<Projectile>,
            Without<Boss>,
            Without<ProjectileHasHit>,
            Without<crate::systems::boss::BossProjectile>,
        ),
    >,
    mut boss_query: Query<(Entity, &Transform, &mut Hp), With<Boss>>,
) {
    const BASE_PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
    const BOSS_SIZE: Vec2 = Vec2::new(32.0, 64.0);

    for (projectile_entity, projectile_transform, projectile) in &projectile_query {
        // Calculate projectile size based on charge level (for collision detection)
        let charge_multiplier = 1.0 + (projectile.charge_level * 1.5);
        let projectile_size = BASE_PROJECTILE_SIZE * charge_multiplier;

        for (boss_entity, boss_transform, mut boss_hp) in &mut boss_query {
            if check_aabb_collision(
                projectile_transform.translation,
                projectile_size,
                boss_transform.translation,
                BOSS_SIZE,
            ) {
                // Calculate damage based on charge level
                // Base damage for uncharged shots, multiplied for charged shots
                let is_charged_shot = projectile.charge_level >= CHARGE_SHOT_MIN_TIME / CHARGE_SHOT_MAX_TIME;
                let damage = if is_charged_shot {
                    // Charged shot: damage scales with charge level
                    let damage_multiplier = 1.0 + (projectile.charge_level * (CHARGE_SHOT_DAMAGE_MULTIPLIER - 1.0));
                    PLAYER_PROJECTILE_DAMAGE * damage_multiplier
                } else {
                    // Normal shot: base damage
                    PLAYER_PROJECTILE_DAMAGE
                };

                // Boss takes damage
                boss_hp.current = (boss_hp.current - damage).max(0.0);

                // Apply knockback to boss if hit by charged shot
                if is_charged_shot {
                    // Knockback direction is the same as projectile direction (pushes boss away from player)
                    let knockback_direction = projectile.direction.normalize_or_zero();
                    commands.entity(boss_entity).insert(Knockback {
                        velocity: knockback_direction * KNOCKBACK_FORCE,
                        timer: KNOCKBACK_DURATION,
                    });
                }

                // Mark projectile as hit (prevents multiple hits before despawn)
                commands.entity(projectile_entity).insert(ProjectileHasHit);

                // Despawn projectile
                commands.entity(projectile_entity).despawn();

                // Only process one collision per projectile
                break;
            }
        }
    }
}

/// System to persist player HP to PlayerUpgrades resource
pub fn persist_player_hp(
    player_query: Query<&Hp, With<Player>>,
    mut player_upgrades: ResMut<PlayerUpgrades>,
) {
    if let Ok(player_hp) = player_query.single() {
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
    _current_stage: ResMut<crate::stages::game_menu::CurrentStage>,
) {
    // Check if player is dead (lose condition)
    if let Ok(player_hp) = player_query.single() {
        if player_hp.current <= 0.0 {
            next_state.set(GameState::GameOver);
            return;
        }
    }

    // Check if boss is dead (win condition)
    if let Ok((boss_hp, boss_type)) = boss_query.single() {
        if boss_hp.current <= 0.0 {
            // Store which boss was defeated
            defeated_boss.boss_type = Some(*boss_type);

            // Always transition to GameWin screen
            // The handle_stage_progression system will check if we should continue to next stage
            next_state.set(GameState::GameWin);
        }
    }
}
