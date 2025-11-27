use crate::components::player::{
    AnimationFrameIndex, AnimationState, AnimationTimer, PlayerAnimationConfig,
};
use bevy::prelude::*;

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationState,
        &PlayerAnimationConfig,
        &mut AnimationFrameIndex,
    )>,
) {
    for (mut timer, mut sprite, state, config, mut frame_index) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            // Get the appropriate frame list based on current state
            let frames = match state {
                AnimationState::Idle => &config.idle,
                AnimationState::Run => &config.run,
                AnimationState::Jump => &config.jump,
                _ => &config.idle,
            };

            // Skip if no frames available
            if frames.is_empty() {
                continue;
            }

            // Advance to next frame
            frame_index.current = (frame_index.current + 1) % frames.len();

            // Update sprite image
            sprite.image = frames[frame_index.current].clone();
        }
    }
}

