use crate::components::player::{
    AnimationState, Dash, JumpCharge, JumpType, Knockback, Player, PlayerVelocity, WallSide,
};
use crate::config::gameplay::{
    BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, KNOCKBACK_DECAY_RATE,
    KNOCKBACK_MOVEMENT_REDUCTION,
};
use crate::config::player::SMALL_JUMP_CHARGE_RATIO;
use bevy::prelude::*;

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
            Option<&mut AnimationState>,
            Option<&mut Sprite>,
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
    const WALL_DETECT_TOLERANCE: f32 = 1.0;
    const WALL_SLIDE_MAX_DESCENT: f32 = -200.0;
    const WALL_JUMP_STRENGTH: f32 = 520.0;
    const WALL_JUMP_HORIZONTAL_SPEED: f32 = 250.0;
    const WALL_JUMP_HORIZONTAL_PUSH_DISTANCE: f32 = 20.0;
    const WALL_DETACH_DURATION: f32 = 0.35;

    for (entity, mut transform, mut velocity, mut jump_charge, dash, knockback, anim_state, mut sprite) in &mut player_query
    {
        if velocity.wall_detach_timer > 0.0 {
            velocity.wall_detach_timer = (velocity.wall_detach_timer - time.delta_secs()).max(0.0);
            if velocity.wall_detach_timer == 0.0 {
                velocity.last_wall_side = None;
            }
        }

        // Movement
        let mut direction = Vec2::ZERO;
        let mut is_moving = false;

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
            is_moving = true;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
            is_moving = true;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        // We don't handle ArrowDown for movement, only for dash
        // if keyboard_input.pressed(KeyCode::ArrowDown) {
        //     direction.y -= 1.0;
        // }

        if velocity.wall_detach_timer > 0.0 {
            if let Some(side) = velocity.last_wall_side {
                let moving_toward_wall = match side {
                    WallSide::Left => direction.x < 0.0,
                    WallSide::Right => direction.x > 0.0,
                };

                let away_dir = match side {
                    WallSide::Left => 1.0,
                    WallSide::Right => -1.0,
                };

                if moving_toward_wall || direction.x.abs() < f32::EPSILON {
                    direction.x = away_dir;
                }
            }
        }

        if direction != Vec2::ZERO {
            velocity.facing_direction = direction.normalize();
        }

        // Update Sprite Flip based on direction
        if let Some(ref mut s) = sprite {
            if velocity.facing_direction.x < 0.0 {
                s.flip_x = true;
            } else if velocity.facing_direction.x > 0.0 {
                s.flip_x = false;
            }
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

        if velocity.wall_jump_velocity.abs() > f32::EPSILON {
            transform.translation.x += velocity.wall_jump_velocity * time.delta_secs();
            // Apply damping to create an arc-like motion
            let damping = 5.0 * time.delta_secs();
            velocity.wall_jump_velocity -= velocity.wall_jump_velocity * damping;
            if velocity.wall_jump_velocity.abs() < 1.0 {
                velocity.wall_jump_velocity = 0.0;
            }
            transform.translation.x = transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
        }

        // Check if jump button is pressed (Space, or X)
        let jump_button_pressed =
            keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::KeyX);
        let jump_button_just_pressed = keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::KeyX);
        let jump_button_just_released = keyboard_input.just_released(KeyCode::Space)
            || keyboard_input.just_released(KeyCode::KeyX);

        let is_on_ground = transform.translation.y <= GROUND_Y;
        let touching_left_wall = transform.translation.x <= BOUNDARY_LEFT + WALL_DETECT_TOLERANCE;
        let touching_right_wall = transform.translation.x >= BOUNDARY_RIGHT - WALL_DETECT_TOLERANCE;

        if is_on_ground {
            velocity.wall_slide = None;
            velocity.can_wall_jump = false;
            velocity.wall_jump_velocity = 0.0;
            velocity.has_wall_jumped = false;
            velocity.wall_detach_timer = 0.0;
            velocity.last_wall_side = None;
        } else if touching_left_wall || touching_right_wall {
            let side = if touching_left_wall {
                WallSide::Left
            } else {
                WallSide::Right
            };

            if velocity.has_wall_jumped {
                velocity.wall_slide = None;
                velocity.can_wall_jump = false;
            } else {
                if velocity.wall_slide != Some(side) {
                    velocity.wall_slide = Some(side);
                    velocity.can_wall_jump = true;
                }

                if velocity.y < WALL_SLIDE_MAX_DESCENT {
                    velocity.y = WALL_SLIDE_MAX_DESCENT;
                }
            }
        } else if velocity.wall_slide.is_some() {
            velocity.wall_slide = None;
            velocity.can_wall_jump = false;
        }

        let mut wall_jump_triggered = false;
        if !is_on_ground && jump_button_just_pressed {
            if let Some(side) = velocity.wall_slide {
                if velocity.can_wall_jump {
                    wall_jump_triggered = true;
                    let horizontal_dir = if side == WallSide::Left { 1.0 } else { -1.0 };
                    velocity.y = WALL_JUMP_STRENGTH;
                    velocity.jump_type = JumpType::High;
                    velocity.wall_slide = None;
                    velocity.can_wall_jump = false;
                    velocity.has_wall_jumped = true;
                    velocity.wall_detach_timer = WALL_DETACH_DURATION;
                    velocity.last_wall_side = Some(side);
                    velocity.facing_direction = Vec2::new(horizontal_dir, 0.0);
                    velocity.wall_jump_velocity = horizontal_dir * WALL_JUMP_HORIZONTAL_SPEED;
                    transform.translation.x += horizontal_dir * WALL_JUMP_HORIZONTAL_PUSH_DISTANCE;
                    transform.translation.x =
                        transform.translation.x.clamp(BOUNDARY_LEFT, BOUNDARY_RIGHT);
                }
            }
        }

        // Dash
        if keyboard_input.pressed(KeyCode::ArrowDown) && jump_button_just_pressed && is_on_ground {
            commands.entity(entity).insert(Dash {
                timer: DASH_DURATION,
                direction: velocity.facing_direction.x,
            });
            return; // No other movement during dash
        }

        if wall_jump_triggered {
            jump_charge.is_charging = false;
            jump_charge.timer = 0.0;
        } else {
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

        // --- Animation State Logic ---
        if let Some(mut state) = anim_state {
            if is_on_ground {
                if is_moving {
                    *state = AnimationState::Run;
                } else {
                    *state = AnimationState::Idle;
                }
            } else {
                *state = AnimationState::Jump;
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
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
}

