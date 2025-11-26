use crate::components::boss::*;
use crate::components::player::Knockback;
use crate::config::gameplay::{
    BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, KNOCKBACK_DECAY_RATE,
};
use bevy::prelude::*;

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
                let center_vec: Vec2 = center.clone().into();
                transform.translation.x = center_vec.x + radius * movement_state.current_angle.cos();
                transform.translation.y = center_vec.y + radius * movement_state.current_angle.sin();

                // Clamp to game boundaries
                transform.translation.x =
                    transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
                transform.translation.y =
                    transform.translation.y.clamp(BOUNDARY_BOTTOM, BOUNDARY_TOP);
            }
            MovementPattern::Waypoint { .. } => {
                // For now, treat waypoint as Stationary - can be extended later
            }
            MovementPattern::Custom => {
                // Custom movement - can be extended
            }
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
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
}
