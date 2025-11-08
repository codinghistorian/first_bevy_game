use bevy::prelude::*;
use crate::components::player::*;
use crate::components::boss::*;
use crate::stages::game_menu::SelectedCharacter;
use crate::systems::config::{SMALL_JUMP_CHARGE_RATIO, KNOCKBACK_FORCE, KNOCKBACK_DURATION, KNOCKBACK_DECAY_RATE, KNOCKBACK_MOVEMENT_REDUCTION, KNOCKBACK_TOP_HORIZONTAL_COMPONENT, KNOCKBACK_TOP_VERTICAL_COMPONENT, KNOCKBACK_SIDE_VERTICAL_COMPONENT};

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
        Hp {
            current: 100.0,
            max: 100.0,
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
        Shooting {
            timer: 0.0,
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

/// Spawns the boss on the right side of the game field
pub fn spawn_boss(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    boss_registry: Option<Res<BossRegistry>>,
) {
    // Get boss data from registry or use default
    let boss_data = boss_registry
        .as_ref()
        .and_then(|registry| registry.get_boss_data(BossType::Default))
        .cloned()
        .unwrap_or_else(|| BossData::default());

    // Spawn the boss character on the right side
    // Position at x = 300 (right side), same y as player (-198)
    let mut boss_entity = commands.spawn((
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
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerVelocity, &mut JumpCharge, Option<&mut Dash>, Option<&Knockback>), With<Player>>,
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

    for (entity, mut transform, mut velocity, mut jump_charge, dash, knockback) in &mut player_query {
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
        // Keep player within screen bounds
        transform.translation.x = transform.translation.x.clamp(-350.0, 350.0);


        // Check if jump button is pressed (Space, or X)
        let jump_button_pressed = keyboard_input.pressed(KeyCode::Space)
            || keyboard_input.pressed(KeyCode::KeyX);
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
    mut player_query: Query<(&Transform, &PlayerVelocity, &mut Shooting), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const SHOOT_COOLDOWN: f32 = 0.5; // Seconds

    for (player_transform, player_velocity, mut shooting) in &mut player_query {
        shooting.timer -= time.delta_secs();

        if keyboard_input.pressed(KeyCode::KeyC) && shooting.timer <= 0.0 {
            // Determine the cardinal shooting direction
            let shoot_direction;

            // Prioritize vertical over horizontal if both are pressed
            if player_velocity.facing_direction.y > 0.0 { // Facing up
                shoot_direction = Vec2::Y;
            } else if player_velocity.facing_direction.x.abs() > 0.0 { // Facing left or right
                shoot_direction = Vec2::X * player_velocity.facing_direction.x.signum();
            } else { // Default to right if no clear direction (e.g., standing still)
                shoot_direction = Vec2::X;
            }

            // Prevent shooting downwards
            if shoot_direction.y < 0.0 {
                return;
            }

            let projectile_transform = Transform::from_xyz(
                player_transform.translation.x,
                player_transform.translation.y,
                0.0,
            );

            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(10.0, 10.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
                projectile_transform,
                Projectile {
                    direction: shoot_direction,
                },
            ));

            shooting.timer = SHOOT_COOLDOWN;
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

        // Despawn projectile after it goes off screen
        if transform.translation.x.abs() > 400.0 || transform.translation.y.abs() > 300.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns the player's HP bar.
pub fn setup_player_hp_bar(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    let Ok(player) = player_query.single() else {
        // Player doesn't exist yet, skip creating HP bar
        return;
    };

    // --- Player HP Bar ---
    // Create a root container that covers the screen
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            // HP bar container positioned at top-left using margins
            parent.spawn((
                Node {
                    width: px(200.0),
                    height: px(30.0),
                    margin: UiRect {
                        left: px(10.0),
                        top: px(10.0),
                        right: px(0.0),
                        bottom: px(0.0),
                    },
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.into()),
            ))
            .with_children(|hp_parent| {
                // HP bar fill
                hp_parent.spawn((
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 1.0, 0.0).into()),
                    HealthBar { entity: player },
                ));
            });
        });
}

/// Spawns the boss's HP bar.
pub fn setup_boss_hp_bar(mut commands: Commands, boss_query: Query<Entity, With<Boss>>) {
    let Ok(boss) = boss_query.single() else {
        // Boss doesn't exist yet, skip creating HP bar
        return;
    };

    // --- Boss HP Bar ---
    // Create a completely separate root container for the boss HP bar
    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            // HP bar container positioned at top-right using margins (separate from player HP bar)
            parent.spawn((
                Node {
                    width: px(200.0),
                    height: px(30.0),
                    margin: UiRect {
                        left: px(0.0),
                        top: px(50.0), // Positioned below player HP bar (10px top + 30px height + 10px gap = 50px)
                        right: px(10.0),
                        bottom: px(0.0),
                    },
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::BLACK.into()),
            ))
            .with_children(|hp_parent| {
                // HP bar fill
                hp_parent.spawn((
                    Node {
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 0.0, 0.0).into()), // Red for boss
                    HealthBar { entity: boss },
                ));
            });
        });
}

/// System to update the width of health bars based on the entity's HP.
pub fn update_health_bars(
    hp_query: Query<&Hp>,
    mut health_bar_query: Query<(&HealthBar, &mut Node)>,
) {
    for (health_bar, mut node) in health_bar_query.iter_mut() {
        if let Ok(hp) = hp_query.get(health_bar.entity) {
            let health_percentage = (hp.current / hp.max) * 100.0;
            node.width = percent(health_percentage);
        }
    }
}

/// System to handle health regeneration (currently disabled - player doesn't regenerate)
/// This can be enabled later if you want health regeneration mechanics
pub fn change_health(
    _time: Res<Time>,
    _player_query: Query<&mut Hp, With<Player>>,
) {
    // Health regeneration disabled - player HP stays at current value
    // Uncomment below to enable regeneration:
    // let mut player_hp = player_query.single_mut().unwrap();
    // player_hp.current = (player_hp.current + 5.0 * time.delta_secs()).min(player_hp.max);
}

/// Helper function to check AABB (Axis-Aligned Bounding Box) collision
fn check_aabb_collision(
    pos1: Vec3,
    size1: Vec2,
    pos2: Vec3,
    size2: Vec2,
) -> bool {
    let half_size1 = size1 * 0.5;
    let half_size2 = size2 * 0.5;
    
    pos1.x - half_size1.x < pos2.x + half_size2.x
        && pos1.x + half_size1.x > pos2.x - half_size2.x
        && pos1.y - half_size1.y < pos2.y + half_size2.y
        && pos1.y + half_size1.y > pos2.y - half_size2.y
}

/// Calculate improved knockback direction based on collision angle
/// This makes knockback feel more dynamic and appropriate for different collision sides
fn calculate_knockback_direction(
    direction_to_player: Vec2,
    player_pos: Vec3,
    boss_pos: Vec3,
) -> Vec2 {
    use crate::systems::config::{KNOCKBACK_TOP_HORIZONTAL_COMPONENT, KNOCKBACK_TOP_VERTICAL_COMPONENT, KNOCKBACK_SIDE_VERTICAL_COMPONENT};
    
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
            ).normalize()
        } else {
            // Player is below boss (hitting from bottom)
            // Push downward and to the side
            let horizontal_dir = if normalized.x > 0.0 { 1.0 } else { -1.0 };
            Vec2::new(
                horizontal_dir * KNOCKBACK_TOP_HORIZONTAL_COMPONENT,
                -KNOCKBACK_TOP_VERTICAL_COMPONENT,
            ).normalize()
        }
    } else {
        // Left or right collision (side collision)
        // Push horizontally away with slight upward component for more dynamic feel
        let horizontal_dir = if normalized.x > 0.0 { 1.0 } else { -1.0 };
        Vec2::new(
            horizontal_dir,
            KNOCKBACK_SIDE_VERTICAL_COMPONENT,
        ).normalize()
    }
}

/// System to handle player-boss collision (player takes damage)
pub fn player_boss_collision(
    time: Res<Time>,
    mut player_query: Query<(Entity, &Transform, &mut Hp, Option<&mut Invincibility>), With<Player>>,
    boss_query: Query<&Transform, With<Boss>>,
    mut commands: Commands,
) {
    const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 64.0);
    const BOSS_SIZE: Vec2 = Vec2::new(32.0, 64.0);
    const DAMAGE: f32 = 10.0;
    const INVINCIBILITY_DURATION: f32 = 1.0; // 1 second of invincibility after taking damage

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
                let direction_to_player = (player_transform.translation - boss_transform.translation).truncate();
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
        
        // Keep player within screen bounds even during knockback
        transform.translation.x = transform.translation.x.clamp(-350.0, 350.0);
        
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
    projectile_query: Query<(Entity, &Transform), (With<Projectile>, Without<Boss>)>,
    mut boss_query: Query<(&Transform, &mut Hp), With<Boss>>,
) {
    const PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
    const BOSS_SIZE: Vec2 = Vec2::new(32.0, 64.0);
    const DAMAGE: f32 = 20.0;

    for (projectile_entity, projectile_transform) in &projectile_query {
        for (boss_transform, mut boss_hp) in &mut boss_query {
            if check_aabb_collision(
                projectile_transform.translation,
                PROJECTILE_SIZE,
                boss_transform.translation,
                BOSS_SIZE,
            ) {
                // Boss takes damage
                boss_hp.current = (boss_hp.current - DAMAGE).max(0.0);
                
                // Despawn projectile
                commands.entity(projectile_entity).despawn();
                
                // Only process one collision per projectile
                break;
            }
        }
    }
}
