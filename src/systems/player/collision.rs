use crate::components::boss::{Boss, BossProjectile};
use crate::components::player::{
    Hp, Invincibility, Knockback, Player, Projectile, ProjectileHasHit,
};
use crate::config::boss::BOSS_COLLISION_DAMAGE;
use crate::config::gameplay::{
    CHARGE_SHOT_DAMAGE_MULTIPLIER, INVINCIBILITY_DURATION, KNOCKBACK_DURATION, KNOCKBACK_FORCE,
    PLAYER_PROJECTILE_DAMAGE,
};
use crate::config::player::{CHARGE_SHOT_MAX_TIME, CHARGE_SHOT_MIN_TIME, CHARGE_SHOT_STRONG_THRESHOLD};
use crate::stages::game_menu::PlayerUpgrades;
use bevy::prelude::*;

/// Helper function to check AABB (Axis-Aligned Bounding Box) collision
pub fn check_aabb_collision(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
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
    _player_pos: Vec3,
    _boss_pos: Vec3,
) -> Vec2 {
    use crate::config::gameplay::{
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
    mut player_query: Query<(Entity, &Transform, &mut Hp, Option<&Invincibility>), With<Player>>,
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
    let damage_amount = BOSS_COLLISION_DAMAGE * defense_multiplier;

    for (player_entity, player_transform, mut player_hp, invincibility) in &mut player_query {
        if invincibility.is_some() {
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
                player_hp.current = (player_hp.current - damage_amount).max(0.0);

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

/// System to handle projectile-boss collision (boss takes damage, projectile despawns)
pub fn projectile_boss_collision(
    mut commands: Commands,
    projectile_query: Query<
        (Entity, &Transform, &Projectile),
        (
            With<Projectile>,
            Without<Boss>,
            Without<ProjectileHasHit>,
            Without<BossProjectile>,
        ),
    >,
    mut boss_query: Query<(Entity, &Transform, &mut Hp, Option<&Invincibility>), With<Boss>>,
) {
    const BASE_PROJECTILE_SIZE: Vec2 = Vec2::new(10.0, 10.0);
    const BOSS_SIZE: Vec2 = Vec2::new(32.0, 64.0);

    for (projectile_entity, projectile_transform, projectile) in &projectile_query {
        // Calculate projectile size based on charge level (for collision detection)
        let charge_multiplier = 1.0 + (projectile.charge_level * 1.5);
        let projectile_size = BASE_PROJECTILE_SIZE * charge_multiplier;

        for (boss_entity, boss_transform, mut boss_hp, invincibility) in &mut boss_query {
            if invincibility.is_some() {
                continue;
            }

            if check_aabb_collision(
                projectile_transform.translation,
                projectile_size,
                boss_transform.translation,
                BOSS_SIZE,
            ) {
                // Calculate damage based on charge level
                // Base damage for uncharged shots, multiplied for charged shots
                let charge_ratio = projectile.charge_level;
                let is_charged_shot = charge_ratio >= CHARGE_SHOT_MIN_TIME / CHARGE_SHOT_MAX_TIME;
                let is_strongest_charge_shot = charge_ratio >= CHARGE_SHOT_STRONG_THRESHOLD;
                let damage = if is_charged_shot {
                    // Charged shot: damage scales with charge level
                    let damage_multiplier =
                        1.0 + (charge_ratio * (CHARGE_SHOT_DAMAGE_MULTIPLIER - 1.0));
                    PLAYER_PROJECTILE_DAMAGE * damage_multiplier
                } else {
                    // Normal shot: base damage
                    PLAYER_PROJECTILE_DAMAGE
                };

                // Boss takes damage
                boss_hp.current = (boss_hp.current - damage).max(0.0);

                // Add invincibility frames
                commands.entity(boss_entity).insert(Invincibility {
                    timer: INVINCIBILITY_DURATION,
                });

                // Apply knockback to boss if hit by charged shot
                if is_strongest_charge_shot {
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

/// System to decrement invincibility timers and clean up when they expire
pub fn update_invincibility_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invincibility, Option<&mut Visibility>)>,
) {
    for (entity, mut invincibility, visibility) in &mut query {
        invincibility.timer -= time.delta_secs();
        if invincibility.timer <= 0.0 {
            commands.entity(entity).remove::<Invincibility>();

            if let Some(mut visibility) = visibility {
                *visibility = Visibility::Visible;
            } else {
                commands.entity(entity).insert(Visibility::Visible);
            }
        }
    }
}

/// System to make invincible entities blink (toggle visibility) while under knockback
/// This provides visual feedback that knockback (and its invincibility window) is active
pub fn invincibility_blink(
    mut commands: Commands,
    mut invincible_query: Query<(Entity, &Invincibility, Option<&mut Visibility>), With<Knockback>>,
) {
    const BLINK_RATE: f32 = 0.1; // Toggle visibility every 0.1 seconds

    for (entity, invincibility, visibility) in &mut invincible_query {
        // Calculate blink state based on invincibility timer
        let blink_cycle = (INVINCIBILITY_DURATION - invincibility.timer) / BLINK_RATE;
        let is_visible = (blink_cycle as i32) % 2 == 0;

        if let Some(mut visibility) = visibility {
            *visibility = if is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        } else {
            // Ensure Visibility component exists
            commands.entity(entity).insert(if is_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            });
        }
    }
}

