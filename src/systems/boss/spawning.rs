use crate::components::boss::{Boss, BossAttackState, BossMovementState};
use crate::components::player::Hp;
use crate::systems::boss::BossPatternRegistry;
use bevy::prelude::*;

/// Spawns the boss on the right side of the game field
pub fn spawn_boss(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    boss_registry: Option<Res<crate::components::boss::BossRegistry>>,
    pattern_registry: Option<Res<BossPatternRegistry>>,
    current_stage: Option<Res<crate::stages::game_menu::CurrentStage>>,
) {
    use crate::components::boss::{BossData, BossType};

    // Get boss data from registry or use default
    let mut boss_data = boss_registry
        .as_ref()
        .and_then(|registry| registry.get_boss_data(BossType::Default))
        .cloned()
        .unwrap_or_else(|| BossData::default());

    let mut boss_hp = 200.0;

    // Try to load pattern from JSON based on stage number
    if let (Some(registry), Some(stage)) = (pattern_registry.as_ref(), current_stage.as_ref()) {
        let stage_num = stage.0;
        let pattern_name = format!("stage_{}", stage_num);

        if let Some(pattern_config) = registry.get_pattern(&pattern_name) {
            // Use loaded patterns directly
            boss_data.attack_pattern = pattern_config.attack.clone();
            boss_data.movement_pattern = pattern_config.movement.clone();
            if let Some(pattern_hp) = pattern_config.hp {
                boss_hp = pattern_hp.max(1.0);
            }
        }
    }

    // Spawn the boss character on the right side
    // Position at x = 300 (right side), same y as player (-198)
    let _boss_entity = commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(boss_data.size.x, boss_data.size.y))),
        MeshMaterial2d(materials.add(boss_data.color)),
        Transform::from_xyz(300.0, -198.0, 1.0), // Positioned on the right side, on top of the floor
        Boss,
        boss_data.boss_type,
        boss_data.clone(),
        Hp {
            current: boss_hp,
            max: boss_hp,
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
