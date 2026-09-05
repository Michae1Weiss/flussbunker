//! The gameplay feature group.
//!
//! Read the list below as the game's architecture: eleven features, each owning
//! its own components, resources and messages. Nothing here knows what is
//! inside them.
//!
//! Note what is *absent*: there is no `reset_game` system. Each feature resets
//! its own state in `OnEnter(Screen::Gameplay)`. A central reset would have to
//! reach into every other feature's resources — that is exactly the coupling
//! this layout exists to prevent.

use bevy::prelude::*;

pub mod bunker;
pub mod combat;
pub mod economy;
pub mod enemy;
pub mod hud;
pub mod map;
pub mod orders;
pub mod projectile;
pub mod tower;
pub mod uboat;
pub mod wave;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        map::plugin,        // static world: tiles, the river, coordinate maths
        combat::plugin,     // shared contract: Health, Armor, Damage, Sunk
        enemy::plugin,      // boats: movement, breaching, depth charges
        tower::plugin,      // shore batteries: construction, targeting, firing
        projectile::plugin, // shells and torpedoes in flight
        uboat::plugin,      // the pen, sorties, torpedoes
        wave::plugin,       // pacing: what spawns, when, and who won
        economy::plugin,    // marks and bounties
        bunker::plugin,     // integrity, i.e. lives
        orders::plugin,     // player intent: what the mouse and keys mean
        hud::plugin,
    ));
}
