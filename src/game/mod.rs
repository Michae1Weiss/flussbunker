//! Gameplay feature plugins. Each feature resets its own state in
//! `OnEnter(Screen::Gameplay)`; there is no central reset.

use bevy::prelude::*;

pub mod art;
pub mod audio;
pub mod bunker;
pub mod combat;
pub mod destruction;
pub mod economy;
pub mod enemy;
pub mod fx;
pub mod hud;
pub mod input;
pub mod juice;
pub mod map;
pub mod notes;
pub mod palette;
pub mod projectile;
pub mod radial;
pub mod telemetry;
pub mod tower;
pub mod uboat;
pub mod wave;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        art::plugin,
        map::plugin,
        combat::plugin,
        destruction::plugin,
        enemy::plugin,
        tower::plugin,
        projectile::plugin,
        uboat::plugin,
        wave::plugin,
    ));
    app.add_plugins((
        economy::plugin,
        bunker::plugin,
        fx::plugin,
        audio::plugin,
        radial::plugin,
        notes::plugin,
        juice::plugin,
        telemetry::plugin,
        input::plugin,
        hud::plugin,
    ));
}
