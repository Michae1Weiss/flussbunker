//! Bunker integrity, the run's lives. Sole writer of `Integrity`; ends the run at 0.

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
        integrity.0 = integrity.0.saturating_sub(breach.damage);
    }

    if integrity.0 == 0 {
        next.set(Screen::Defeat);
    }
}
