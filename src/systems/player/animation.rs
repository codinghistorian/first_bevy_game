use crate::components::player::{
    AnimationState, AnimationTimer, PlayerAnimationConfig,
};
use bevy::prelude::*;

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationState,
        &PlayerAnimationConfig,
    )>,
) {
    for (mut timer, mut sprite, state, config) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let indices = match state {
                AnimationState::Idle => &config.idle,
                AnimationState::Run => &config.run,
                AnimationState::Jump => &config.jump,
                _ => &config.idle,
            };

            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index < indices.first || atlas.index > indices.last {
                    atlas.index = indices.first;
                } else {
                    atlas.index += 1;
                    if atlas.index > indices.last {
                        atlas.index = indices.first;
                    }
                }
            }
        }
    }
}

