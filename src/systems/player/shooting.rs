use crate::components::player::{
    ChargeEffect, ChargeShot, Player, PlayerVelocity, Projectile, Shooting, WallSide,
};
use crate::config::gameplay::{
    BOUNDARY_BOTTOM, BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP,
};
use crate::config::player::{
    CHARGE_SHOT_COOLDOWN, CHARGE_SHOT_MAX_TIME, CHARGE_SHOT_MIN_TIME, NORMAL_SHOT_COOLDOWN,
};
use crate::stages::game_menu::SelectedCharacter;
use bevy::prelude::*;

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
            // Wall slide shooting overrides other directions
            if let Some(wall_side) = player_velocity.wall_slide {
                return match wall_side {
                    WallSide::Left => Some(Vec2::X),  // Shoot right
                    WallSide::Right => Some(-Vec2::X), // Shoot left
                };
            }

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
                    ChargeEffect { player_entity },
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
    mut charge_effect_query: Query<
        (
            &ChargeEffect,
            &mut Transform,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        Without<Player>,
    >,
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

