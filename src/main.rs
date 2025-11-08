use bevy::prelude::*;

mod components;
mod plugins;
mod stages;
mod systems;

use stages::game_menu::{GameMenuPlugin, GameState, SelectedCharacter};
use plugins::player_plugin::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_resource::<SelectedCharacter>()
        .add_plugins(GameMenuPlugin)
        .add_plugins(PlayerPlugin)
        .run();
}
