//! Screen flow: `Screen` is where the game is, `Menu` which paper menu lies on top of it.
//! `DespawnOnExit` does all cleanup.

use bevy::prelude::*;

mod game_over;
mod pause;
mod settings;
mod title;
pub mod widget;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Title,
    Gameplay,
    Victory,
    Defeat,
}

/// The menu on top of the screen. The world takes input only under `Menu::None`.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Menu {
    #[default]
    None,
    Main,
    Pause,
    Settings,
}

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>().add_plugins((
        title::plugin,
        pause::plugin,
        settings::plugin,
        game_over::plugin,
        widget::plugin,
    ));
}
