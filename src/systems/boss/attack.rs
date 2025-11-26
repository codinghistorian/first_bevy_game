use crate::components::boss::*;
use crate::components::player::*;
use crate::config::gameplay::{
    BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, KNOCKBACK_DURATION,
    KNOCKBACK_FORCE,
};
use bevy::prelude::*;

/// Helper function to snap a direction vector to cardinal directions (horizontal or vertical only)
/// Returns a normalized vector pointing either horizontally or vertically, whichever is closer
fn snap_to_cardinal(direction: Vec2) -> Vec2 {
    if direction.length() < 0.001 {
        // If direction is zero or very small, default to left (toward player)
        return Vec2::new(-1.0, 0.0);
    }

    let abs_x = direction.x.abs();
    let abs_y = direction.y.abs();

    // Choose the axis with the larger component
    if abs_x > abs_y {
        // Horizontal direction
        Vec2::new(direction.x.signum(), 0.0)
    } else {
        // Vertical direction
        Vec2::new(0.0, direction.y.signum())
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
            charge_level: 0.0, // Boss projectiles are always uncharged
        },
        BossProjectile {
            speed: velocity.length(),
            damage: crate::config::boss::BOSS_PROJECTILE_DAMAGE, // Default damage
        },
    ));
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
                cardinal_only,
            } => {
                if attack_state.timer <= 0.0 {
                    // Get player position for aiming
                    if let Some(player_transform) = player_query.iter().next() {
                        let mut direction = (player_transform.translation
                            - boss_transform.translation)
                            .truncate()
                            .normalize_or_zero();

                        // Snap to cardinal directions if enabled
                        if *cardinal_only {
                            direction = snap_to_cardinal(direction);
                        }

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
                    if let Some(player_transform) = player_query.iter().next() {
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
                        if let Some(player_transform) = player_query.iter().next() {
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
            AttackPattern::Sequence { .. } => {
                // Sequence handling implementation would go here
            }
            AttackPattern::Custom { cooldown: _ } => {
                // Custom attack pattern - can be extended
            }
        }
    }
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
        (Entity, &Transform, &Projectile, &BossProjectile),
        (With<BossProjectile>, Without<Player>),
    >,
    mut player_query: Query<(Entity, &Transform, &mut Hp, Option<&Invincibility>), With<Player>>,
    player_upgrades: Option<Res<crate::stages::game_menu::PlayerUpgrades>>,
) {
    use crate::config::gameplay::INVINCIBILITY_DURATION;
    use crate::systems::player::check_aabb_collision;

    const PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
    const PLAYER_SIZE: Vec2 = Vec2::new(32.0, 64.0);

    // Apply defense multiplier to damage
    let defense_multiplier = player_upgrades
        .as_ref()
        .map(|u| u.defense_multiplier)
        .unwrap_or(1.0);

    for (projectile_entity, projectile_transform, projectile, boss_projectile) in &projectile_query {
        let damage = boss_projectile.damage * defense_multiplier;

        for (player_entity, player_transform, mut player_hp, invincibility) in &mut player_query {
            if invincibility.is_some() {
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
                player_hp.current = (player_hp.current - damage).max(0.0);

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

