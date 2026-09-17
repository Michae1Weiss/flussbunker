//! Gameplay feature plugins. Each feature resets its own state in
//! `OnEnter(Screen::Gameplay)`; there is no central reset.

use bevy::prelude::*;

pub mod bunker;
pub mod combat;
pub mod economy;
pub mod enemy;
pub mod hud;
pub mod input;
pub mod map;
pub mod projectile;
pub mod tower;
pub mod uboat;
pub mod wave;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        map::plugin,
        combat::plugin,
        enemy::plugin,
        tower::plugin,
        projectile::plugin,
        uboat::plugin,
        wave::plugin,
        economy::plugin,
        bunker::plugin,
        input::plugin,
        hud::plugin,
    ));
}
