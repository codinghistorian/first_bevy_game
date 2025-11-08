use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::boss::*;
use crate::components::player::*;

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
    pub action_type: String, // "shoot", "wait", "burst", etc.
    pub direction: Option<Vec2Config>, // Direction to shoot
    pub count: Option<u32>, // Number of shots
    pub delay: Option<f32>, // Delay before next action
    pub spread: Option<f32>, // Spread angle for multi-shot
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

    /// Get a pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&BossPatternConfig> {
        self.patterns.get(name)
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
            MovementPattern::HorizontalPatrol { left_bound, right_bound, speed } => {
                // Move horizontally between bounds
                transform.translation.x += movement_state.direction * speed * time.delta_secs();
                
                // Reverse direction at bounds
                if transform.translation.x <= *left_bound {
                    transform.translation.x = *left_bound;
                    movement_state.direction = 1.0;
                } else if transform.translation.x >= *right_bound {
                    transform.translation.x = *right_bound;
                    movement_state.direction = -1.0;
                }
            }
            MovementPattern::VerticalPatrol { top_bound, bottom_bound, speed } => {
                // Move vertically between bounds
                transform.translation.y += movement_state.direction * speed * time.delta_secs();
                
                // Reverse direction at bounds
                if transform.translation.y <= *bottom_bound {
                    transform.translation.y = *bottom_bound;
                    movement_state.direction = 1.0;
                } else if transform.translation.y >= *top_bound {
                    transform.translation.y = *top_bound;
                    movement_state.direction = -1.0;
                }
            }
            MovementPattern::Circular { center, radius, speed } => {
                // Circular movement
                movement_state.current_angle += speed * time.delta_secs();
                transform.translation.x = center.x + radius * movement_state.current_angle.cos();
                transform.translation.y = center.y + radius * movement_state.current_angle.sin();
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
            AttackPattern::SingleShot { cooldown, projectile_speed } => {
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
            AttackPattern::TripleShot { cooldown, projectile_speed, spread_angle } => {
                if attack_state.timer <= 0.0 {
                    if let Ok(player_transform) = player_query.single() {
                        let base_direction = (player_transform.translation - boss_transform.translation)
                            .truncate()
                            .normalize_or_zero();
                        
                        // Shoot three projectiles with spread
                        let angles = [-*spread_angle, 0.0, *spread_angle];
                        for angle in angles {
                            let rotation = angle.to_radians();
                            let direction = Vec2::new(
                                base_direction.x * rotation.cos() - base_direction.y * rotation.sin(),
                                base_direction.x * rotation.sin() + base_direction.y * rotation.cos(),
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
            AttackPattern::RapidFire { cooldown, projectile_speed, burst_count, burst_delay } => {
                if attack_state.burst_count > 0 {
                    // In burst mode
                    attack_state.burst_timer -= time.delta_secs();
                    if attack_state.burst_timer <= 0.0 {
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
        transform.translation.x += projectile.direction.x * boss_projectile.speed * time.delta_secs();
        transform.translation.y += projectile.direction.y * boss_projectile.speed * time.delta_secs();

        // Despawn projectile after it goes off screen
        if transform.translation.x.abs() > 400.0 || transform.translation.y.abs() > 300.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle boss projectile collision with player
pub fn boss_projectile_player_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), (With<BossProjectile>, Without<Player>)>,
    mut player_query: Query<(Entity, &Transform, &mut Hp, Option<&mut Invincibility>), With<Player>>,
    time: Res<Time>,
) {
    const PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
    const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 64.0);
    const DAMAGE: f32 = 15.0;
    const INVINCIBILITY_DURATION: f32 = 0.5;

    for (projectile_entity, projectile_transform) in &projectile_query {
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

            // Check collision
            let half_projectile = PROJECTILE_SIZE * 0.5;
            let half_player = PLAYER_SIZE * 0.5;
            
            if projectile_transform.translation.x - half_projectile.x < player_transform.translation.x + half_player.x
                && projectile_transform.translation.x + half_projectile.x > player_transform.translation.x - half_player.x
                && projectile_transform.translation.y - half_projectile.y < player_transform.translation.y + half_player.y
                && projectile_transform.translation.y + half_projectile.y > player_transform.translation.y - half_player.y
            {
                // Player takes damage
                player_hp.current = (player_hp.current - DAMAGE).max(0.0);
                
                // Add invincibility frames
                commands.entity(player_entity).insert(Invincibility {
                    timer: INVINCIBILITY_DURATION,
                });
                
                // Despawn projectile
                commands.entity(projectile_entity).despawn();
                
                // Only process one collision per projectile
                break;
            }
        }
    }
}

