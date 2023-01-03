use bevy::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

mod plugin;
mod talkie_core;

#[derive(Component)]
struct GameCamera;

/// Spawn the camera
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(GameCamera);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_fixed_timestep(
            Duration::from_millis(125),
            // give it a label
            "my_fixed_update",
        )
        .add_plugin(plugin::TalkiePlugin)
        // setup our camera globally (for UI) at startup and keep it alive at all times
        .add_startup_system(setup_camera)
        .run();
}
