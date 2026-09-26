//! Flussbunker: a river tower defense with one player-commanded U-boat.
//! `GamePlugin` is the whole interface; everything else is a feature plugin under `game`.

pub mod balance;
mod dev_tools;
pub mod game;
mod game_plugin;
pub mod screens;
pub mod settings;

pub use game_plugin::{harness, GamePlugin};
