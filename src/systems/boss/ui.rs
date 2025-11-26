use crate::components::boss::{Boss, BossHealthBarContainer};
use crate::components::player::HealthBar;
use crate::config::ui::{
    BOSS_HP_BAR_HEIGHT, BOSS_HP_BAR_MARGIN_BOTTOM, BOSS_HP_BAR_MARGIN_LEFT,
    BOSS_HP_BAR_MARGIN_RIGHT, BOSS_HP_BAR_MARGIN_TOP, BOSS_HP_BAR_USE_CENTER, BOSS_HP_BAR_WIDTH,
};
use bevy::prelude::*;

/// Spawns the boss's HP bar.
pub fn setup_boss_hp_bar(mut commands: Commands, boss_query: Query<Entity, With<Boss>>) {
    let Some(boss) = boss_query.iter().next() else {
        // Boss doesn't exist yet, skip creating HP bar
        return;
    };

    // --- Boss HP Bar ---
    // Create a completely separate root container for the boss HP bar
    let root_node = if BOSS_HP_BAR_USE_CENTER {
        // Use center alignment
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        }
    } else {
        // Use margin-based positioning with horizontal centering
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center, // Center horizontally
            align_items: AlignItems::FlexStart, // Align to top for margin-based vertical positioning
            ..default()
        }
    };

    commands
        .spawn((root_node, BossHealthBarContainer))
        .with_children(|parent| {
            // HP bar container with configurable positioning
            let hp_bar_node = if BOSS_HP_BAR_USE_CENTER {
                // Centered - no margins needed
                Node {
                    width: Val::Px(BOSS_HP_BAR_WIDTH),
                    height: Val::Px(BOSS_HP_BAR_HEIGHT),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                }
            } else {
                // Margin-based positioning
                Node {
                    width: Val::Px(BOSS_HP_BAR_WIDTH),
                    height: Val::Px(BOSS_HP_BAR_HEIGHT),
                    margin: UiRect {
                        left: Val::Px(BOSS_HP_BAR_MARGIN_LEFT),
                        top: Val::Px(BOSS_HP_BAR_MARGIN_TOP),
                        right: Val::Px(BOSS_HP_BAR_MARGIN_RIGHT),
                        bottom: Val::Px(BOSS_HP_BAR_MARGIN_BOTTOM),
                    },
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                }
            };

            parent
                .spawn((hp_bar_node, BackgroundColor(Color::BLACK.into())))
                .with_children(|hp_parent| {
                    // HP bar fill
                    hp_parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(1.0, 0.0, 0.0).into()), // Red for boss
                        HealthBar { entity: boss },
                    ));
                });
        });
}

