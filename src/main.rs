use bevy::prelude::*;

mod plugins;
mod systems;
mod components;

use plugins::hello_plugins::HelloPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}