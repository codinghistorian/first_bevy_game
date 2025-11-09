use crate::components::boss::*;
use crate::components::player::*;
use crate::systems::config::{
    BOSS_HP_BAR_HEIGHT, BOSS_HP_BAR_MARGIN_BOTTOM, BOSS_HP_BAR_MARGIN_LEFT,
    BOSS_HP_BAR_MARGIN_RIGHT, BOSS_HP_BAR_MARGIN_TOP, BOSS_HP_BAR_USE_CENTER, BOSS_HP_BAR_WIDTH,
    BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, KNOCKBACK_DURATION,
    KNOCKBACK_FORCE,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// JSON structure for boss attack patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossPatternConfig {
    pub attack: AttackPatternConfig,
    pub movement: MovementPatternConfig,
}

/// JSON structure for attack patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AttackPatternConfig {
    None,
    SingleShot {
        cooldown: f32,
        projectile_speed: f32,
    },
    TripleShot {
        cooldown: f32,
        projectile_speed: f32,
        spread_angle: f32,
    },
    RapidFire {
        cooldown: f32,
        projectile_speed: f32,
        burst_count: u32,
        burst_delay: f32,
    },
    /// Pattern with multiple actions in sequence
    Sequence {
        actions: Vec<AttackAction>,
        loop_pattern: bool,
    },
}

/// Individual attack action in a sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackAction {
    pub action_type: String,           // "shoot", "wait", "burst", etc.
    pub direction: Option<Vec2Config>, // Direction to shoot
    pub count: Option<u32>,            // Number of shots
    pub delay: Option<f32>,            // Delay before next action
    pub spread: Option<f32>,           // Spread angle for multi-shot
}

/// Vec2 configuration for JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec2Config {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2Config> for Vec2 {
    fn from(v: Vec2Config) -> Self {
        Vec2::new(v.x, v.y)
    }
}

/// JSON structure for movement patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MovementPatternConfig {
    Stationary,
    HorizontalPatrol {
        left_bound: f32,
        right_bound: f32,
        speed: f32,
    },
    VerticalPatrol {
        top_bound: f32,
        bottom_bound: f32,
        speed: f32,
    },
    Circular {
        center: Vec2Config,
        radius: f32,
        speed: f32,
    },
    /// Pattern with multiple waypoints
    Waypoint {
        waypoints: Vec<Vec2Config>,
        speed: f32,
        loop_path: bool,
    },
}

/// Resource to store loaded boss patterns from JSON
#[derive(Resource, Default)]
pub struct BossPatternRegistry {
    pub patterns: std::collections::HashMap<String, BossPatternConfig>,
}

impl BossPatternRegistry {
    /// Load a pattern from a JSON string
    pub fn load_from_json(&mut self, name: String, json: &str) -> Result<(), serde_json::Error> {
        let pattern: BossPatternConfig = serde_json::from_str(json)?;
        self.patterns.insert(name, pattern);
        Ok(())
    }

    /// Load a pattern from a JSON file path
    pub fn load_from_file(
        &mut self,
        name: String,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(file_path)?;
        self.load_from_json(name, &json)?;
        Ok(())
    }

    /// Get a pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&BossPatternConfig> {
        self.patterns.get(name)
    }
}

/// Convert JSON attack pattern config to internal AttackPattern
pub fn convert_attack_pattern(config: &AttackPatternConfig) -> AttackPattern {
    match config {
        AttackPatternConfig::None => AttackPattern::None,
        AttackPatternConfig::SingleShot {
            cooldown,
            projectile_speed,
        } => AttackPattern::SingleShot {
            cooldown: *cooldown,
            projectile_speed: *projectile_speed,
        },
        AttackPatternConfig::TripleShot {
            cooldown,
            projectile_speed,
            spread_angle,
        } => AttackPattern::TripleShot {
            cooldown: *cooldown,
            projectile_speed: *projectile_speed,
            spread_angle: *spread_angle,
        },
        AttackPatternConfig::RapidFire {
            cooldown,
            projectile_speed,
            burst_count,
            burst_delay,
        } => AttackPattern::RapidFire {
            cooldown: *cooldown,
            projectile_speed: *projectile_speed,
            burst_count: *burst_count,
            burst_delay: *burst_delay,
        },
        AttackPatternConfig::Sequence { .. } => {
            // For now, treat sequence as None - can be extended later
            AttackPattern::None
        }
    }
}

/// Convert JSON movement pattern config to internal MovementPattern
pub fn convert_movement_pattern(config: &MovementPatternConfig) -> MovementPattern {
    match config {
        MovementPatternConfig::Stationary => MovementPattern::Stationary,
        MovementPatternConfig::HorizontalPatrol {
            left_bound,
            right_bound,
            speed,
        } => MovementPattern::HorizontalPatrol {
            left_bound: *left_bound,
            right_bound: *right_bound,
            speed: *speed,
        },
        MovementPatternConfig::VerticalPatrol {
            top_bound,
            bottom_bound,
            speed,
        } => MovementPattern::VerticalPatrol {
            top_bound: *top_bound,
            bottom_bound: *bottom_bound,
            speed: *speed,
        },
        MovementPatternConfig::Circular {
            center,
            radius,
            speed,
        } => MovementPattern::Circular {
            center: center.clone().into(),
            radius: *radius,
            speed: *speed,
        },
        MovementPatternConfig::Waypoint { .. } => {
            // For now, treat waypoint as Stationary - can be extended later
            MovementPattern::Stationary
        }
    }
}

/// System to load boss pattern for the current stage
pub fn load_stage_boss_pattern(
    mut pattern_registry: ResMut<BossPatternRegistry>,
    current_stage: Res<crate::stages::game_menu::CurrentStage>,
) {
    let stage_num = current_stage.0;
    let pattern_name = format!("stage_{}", stage_num);
    let file_path = format!("boss_patterns/stage_{}_boss.json", stage_num);

    // Only load if not already loaded
    if pattern_registry.get_pattern(&pattern_name).is_none() {
        if let Err(e) = pattern_registry.load_from_file(pattern_name.clone(), &file_path) {
            eprintln!(
                "Warning: Failed to load boss pattern from {}: {}",
                file_path, e
            );
            eprintln!("Using default boss pattern instead");
        }
    }
}

/// System to handle boss movement based on pattern
pub fn boss_movement(
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &BossData, &mut BossMovementState), With<Boss>>,
) {
    for (mut transform, boss_data, mut movement_state) in &mut boss_query {
        match &boss_data.movement_pattern {
            MovementPattern::Stationary => {
                // Boss doesn't move
            }
            MovementPattern::HorizontalPatrol {
                left_bound,
                right_bound,
                speed,
            } => {
                // Move horizontally between bounds
                transform.translation.x += movement_state.direction * speed * time.delta_secs();

                // Clamp to game boundaries first
                transform.translation.x =
                    transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
                transform.translation.y =
                    transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);

                // Reverse direction at bounds
                let effective_left = left_bound.max(BOUNDARY_LEFT);
                let effective_right = right_bound.min(BOUNDARY_RIGHT);
                if transform.translation.x <= effective_left {
                    transform.translation.x = effective_left;
                    movement_state.direction = 1.0;
                } else if transform.translation.x >= effective_right {
                    transform.translation.x = effective_right;
                    movement_state.direction = -1.0;
                }
            }
            MovementPattern::VerticalPatrol {
                top_bound,
                bottom_bound,
                speed,
            } => {
                // Move vertically between bounds
                transform.translation.y += movement_state.direction * speed * time.delta_secs();

                // Clamp to game boundaries first
                transform.translation.x =
                    transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
                transform.translation.y =
                    transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);

                // Reverse direction at bounds
                let effective_bottom = bottom_bound.max(BOUNDARY_BOTTOM);
                let effective_top = top_bound.min(BOUNDARY_TOP);
                if transform.translation.y <= effective_bottom {
                    transform.translation.y = effective_bottom;
                    movement_state.direction = 1.0;
                } else if transform.translation.y >= effective_top {
                    transform.translation.y = effective_top;
                    movement_state.direction = -1.0;
                }
            }
            MovementPattern::Circular {
                center,
                radius,
                speed,
            } => {
                // Circular movement
                movement_state.current_angle += speed * time.delta_secs();
                transform.translation.x = center.x + radius * movement_state.current_angle.cos();
                transform.translation.y = center.y + radius * movement_state.current_angle.sin();

                // Clamp to game boundaries
                transform.translation.x =
                    transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
                transform.translation.y =
                    transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);
            }
            MovementPattern::Custom => {
                // Custom movement - can be extended
            }
        }
    }
}

/// System to handle boss attacks based on pattern
pub fn boss_attacks(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut boss_query: Query<(&Transform, &BossData, &mut BossAttackState), With<Boss>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
) {
    for (boss_transform, boss_data, mut attack_state) in &mut boss_query {
        attack_state.timer -= time.delta_secs();

        match &boss_data.attack_pattern {
            AttackPattern::None => {
                // Boss doesn't attack
            }
            AttackPattern::SingleShot {
                cooldown,
                projectile_speed,
            } => {
                if attack_state.timer <= 0.0 {
                    // Get player position for aiming
                    if let Ok(player_transform) = player_query.single() {
                        let direction = (player_transform.translation - boss_transform.translation)
                            .truncate()
                            .normalize_or_zero();

                        spawn_boss_projectile(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            boss_transform.translation,
                            direction * *projectile_speed,
                        );

                        attack_state.timer = *cooldown;
                    }
                }
            }
            AttackPattern::TripleShot {
                cooldown,
                projectile_speed,
                spread_angle,
            } => {
                if attack_state.timer <= 0.0 {
                    if let Ok(player_transform) = player_query.single() {
                        let base_direction = (player_transform.translation
                            - boss_transform.translation)
                            .truncate()
                            .normalize_or_zero();

                        // Shoot three projectiles with spread
                        let angles = [-*spread_angle, 0.0, *spread_angle];
                        for angle in angles {
                            let rotation = angle.to_radians();
                            let direction = Vec2::new(
                                base_direction.x * rotation.cos()
                                    - base_direction.y * rotation.sin(),
                                base_direction.x * rotation.sin()
                                    + base_direction.y * rotation.cos(),
                            );

                            spawn_boss_projectile(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                boss_transform.translation,
                                direction * *projectile_speed,
                            );
                        }

                        attack_state.timer = *cooldown;
                    }
                }
            }
            AttackPattern::RapidFire {
                cooldown,
                projectile_speed,
                burst_count,
                burst_delay,
            } => {
                if attack_state.burst_count > 0 {
                    // In burst mode
                    attack_state.burst_timer -= time.delta_secs();
                    if attack_state.burst_timer <= 0.0 {
                        if let Ok(player_transform) = player_query.single() {
                            let direction = (player_transform.translation
                                - boss_transform.translation)
                                .truncate()
                                .normalize_or_zero();

                            spawn_boss_projectile(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                boss_transform.translation,
                                direction * *projectile_speed,
                            );

                            attack_state.burst_count -= 1;
                            if attack_state.burst_count > 0 {
                                attack_state.burst_timer = *burst_delay;
                            } else {
                                attack_state.timer = *cooldown;
                            }
                        }
                    }
                } else if attack_state.timer <= 0.0 {
                    // Start new burst
                    attack_state.burst_count = *burst_count;
                    attack_state.burst_timer = *burst_delay;
                }
            }
            AttackPattern::Custom { cooldown: _ } => {
                // Custom attack pattern - can be extended
            }
        }
    }
}

/// Helper function to spawn a boss projectile
fn spawn_boss_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec3,
    velocity: Vec2,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(10.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.0))), // Orange boss projectiles
        Transform::from_xyz(position.x, position.y, 0.0),
        Projectile {
            direction: velocity.normalize_or_zero(),
        },
        BossProjectile {
            speed: velocity.length(),
        },
    ));
}

/// Marker component for boss projectiles (to distinguish from player projectiles)
#[derive(Component)]
pub struct BossProjectile {
    pub speed: f32,
}

/// System to move boss projectiles
pub fn boss_projectile_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &Projectile, &BossProjectile)>,
) {
    for (entity, mut transform, projectile, boss_projectile) in &mut projectile_query {
        transform.translation.x +=
            projectile.direction.x * boss_projectile.speed * time.delta_secs();
        transform.translation.y +=
            projectile.direction.y * boss_projectile.speed * time.delta_secs();

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

/// System to handle boss projectile collision with player
pub fn boss_projectile_player_collision(
    mut commands: Commands,
    projectile_query: Query<
        (Entity, &Transform, &Projectile),
        (With<BossProjectile>, Without<Player>),
    >,
    mut player_query: Query<
        (Entity, &Transform, &mut Hp, Option<&mut Invincibility>),
        With<Player>,
    >,
    time: Res<Time>,
    player_upgrades: Option<Res<crate::stages::game_menu::PlayerUpgrades>>,
) {
    use crate::systems::config::INVINCIBILITY_DURATION;
    use crate::systems::player::check_aabb_collision;

    const PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
    const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 64.0);
    const BASE_DAMAGE: f32 = 15.0;

    // Apply defense multiplier to damage
    let defense_multiplier = player_upgrades
        .as_ref()
        .map(|u| u.defense_multiplier)
        .unwrap_or(1.0);
    let DAMAGE = BASE_DAMAGE * defense_multiplier;

    for (projectile_entity, projectile_transform, projectile) in &projectile_query {
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

            // Check collision using the same AABB function as other collisions
            if check_aabb_collision(
                projectile_transform.translation,
                PROJECTILE_SIZE,
                player_transform.translation,
                PLAYER_SIZE,
            ) {
                // Calculate knockback direction: push player away from the boss (same direction as projectile was traveling)
                // The projectile direction points from boss toward player, so we use the same direction
                // to push the player further away from the boss
                let knockback_direction = projectile.direction.normalize_or_zero();

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

                // Despawn projectile
                commands.entity(projectile_entity).despawn();

                // Only process one collision per projectile
                break;
            }
        }
    }
}

/// Spawns the boss's HP bar.
pub fn setup_boss_hp_bar(mut commands: Commands, boss_query: Query<Entity, With<Boss>>) {
    let Ok(boss) = boss_query.single() else {
        // Boss doesn't exist yet, skip creating HP bar
        return;
    };

    // --- Boss HP Bar ---
    // Create a completely separate root container for the boss HP bar
    let root_node = if BOSS_HP_BAR_USE_CENTER {
        // Use center alignment
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }
    } else {
        // Use margin-based positioning with horizontal centering
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center, // Center horizontally
            align_items: AlignItems::FlexStart, // Align to top for margin-based vertical positioning
            ..default()
        }
    };

    commands.spawn((root_node, BossHealthBarContainer)).with_children(|parent| {
        // HP bar container with configurable positioning
        let hp_bar_node = if BOSS_HP_BAR_USE_CENTER {
            // Centered - no margins needed
            Node {
                width: px(BOSS_HP_BAR_WIDTH),
                height: px(BOSS_HP_BAR_HEIGHT),
                border: UiRect::all(px(2.0)),
                ..default()
            }
        } else {
            // Margin-based positioning
            Node {
                width: px(BOSS_HP_BAR_WIDTH),
                height: px(BOSS_HP_BAR_HEIGHT),
                margin: UiRect {
                    left: px(BOSS_HP_BAR_MARGIN_LEFT),
                    top: px(BOSS_HP_BAR_MARGIN_TOP),
                    right: px(BOSS_HP_BAR_MARGIN_RIGHT),
                    bottom: px(BOSS_HP_BAR_MARGIN_BOTTOM),
                },
                border: UiRect::all(px(2.0)),
                ..default()
            }
        };

        parent
            .spawn((hp_bar_node, BackgroundColor(Color::BLACK.into())))
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
