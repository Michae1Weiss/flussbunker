//! Bunker integrity — this game's "lives".
//!
//! Small on purpose. It reads one message, subtracts, and loses the game at
//! zero. The temptation is to fold this into a `GameState` resource alongside
//! money and wave number; resist it. Three focused resources can be changed
//! independently, while one god-resource means every system that touches money
//! also gets mutable access to your lives.

use bevy::prelude::*;

use crate::balance::STARTING_INTEGRITY;
use crate::game::enemy::EnemyReachedBunker;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Integrity>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(FixedUpdate, take_breaches.in_set(Sim::Death));
}

#[derive(Resource, Debug)]
pub struct Integrity(pub u32);

impl Default for Integrity {
    fn default() -> Self {
        Self(STARTING_INTEGRITY)
    }
}

fn reset(mut integrity: ResMut<Integrity>) {
    *integrity = Integrity::default();
}

fn take_breaches(
    mut breaches: MessageReader<EnemyReachedBunker>,
    mut integrity: ResMut<Integrity>,
    mut next: ResMut<NextState<Screen>>,
) {
    for breach in breaches.read() {
        // `saturating_sub`: an unsigned underflow here would wrap to four
        // billion integrity and silently make the game unlosable. Defining the
        // error out of existence beats asserting on it.
        integrity.0 = integrity.0.saturating_sub(breach.damage);
    }

    if integrity.0 == 0 {
        next.set(Screen::Defeat);
    }
}
