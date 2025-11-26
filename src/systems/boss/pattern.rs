use crate::components::boss::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// JSON structure for boss attack patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossPatternConfig {
    #[serde(default)]
    pub hp: Option<f32>,
    pub attack: AttackPattern,
    pub movement: MovementPattern,
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

    /// Load a pattern from a JSON file path
    pub fn load_from_file(
        &mut self,
        name: String,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(file_path)?;
        self.load_from_json(name, &json)?;
        Ok(())
    }

    /// Get a pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&BossPatternConfig> {
        self.patterns.get(name)
    }
}

/// System to load boss pattern for the current stage
pub fn load_stage_boss_pattern(
    mut pattern_registry: ResMut<BossPatternRegistry>,
    current_stage: Res<crate::stages::game_menu::CurrentStage>,
) {
    let stage_num = current_stage.0;
    let pattern_name = format!("stage_{}", stage_num);
    let file_path = format!("boss_patterns/stage_{}_boss.json", stage_num);

    // Only load if not already loaded
    if pattern_registry.get_pattern(&pattern_name).is_none() {
        if let Err(e) = pattern_registry.load_from_file(pattern_name.clone(), &file_path) {
            eprintln!(
                "Warning: Failed to load boss pattern from {}: {}",
                file_path, e
            );
            eprintln!("Using default boss pattern instead");
        }
    }
}

