//! Bunker Valentin — a 2D tower defense on the Weser, in Bevy 0.19.
//!
//! `main` does two things: configure the window, add `GamePlugin`. That is the
//! whole public interface of the game. Everything else is hidden behind that
//! one line — the plugin is the deep module.

use bevy::{prelude::*, window::WindowResolution};

mod balance;
mod dev_tools;
mod game;
mod game_plugin;
mod screens;

use crate::game_plugin::GamePlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bunker Valentin".to_string(),
                        resolution: WindowResolution::new(960, 720),
                        ..default()
                    }),
                    ..default()
                })
                // Crisp pixels for flat-colour sprites; irrelevant now, correct later.
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(GamePlugin)
        .run()
}
