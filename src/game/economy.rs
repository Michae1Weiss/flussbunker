//! Money. One resource and one observer.
//!
//! `Treasury` is a `Resource` because it is durable state — the classic mistake
//! is modelling money as a stream of `MoneyChanged` messages and then having to
//! reconstruct the balance. Messages describe *events*; resources hold *state*.
//!
//! Dependency direction worth noticing: economy reads `enemy::Bounty`, so
//! economy depends on enemy. Enemy does not know economy exists. Money can be
//! deleted from this game without touching a single enemy system.

use bevy::prelude::*;

use crate::balance::STARTING_MARKS;
use crate::game::combat::Sunk;
use crate::game::enemy::Bounty;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Treasury>()
        // Each feature resets its own state. No central reset system.
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_observer(award_bounty);
}

#[derive(Resource, Debug)]
pub struct Treasury {
    pub marks: u32,
}

impl Default for Treasury {
    fn default() -> Self {
        Self { marks: STARTING_MARKS }
    }
}

impl Treasury {
    /// Spend if affordable, report whether it happened.
    ///
    /// A single checked verb instead of `can_afford()` + `deduct()` at every
    /// call site: two calls that must always appear together are one operation
    /// wearing a disguise, and the split version invites the bug where somebody
    /// forgets the check.
    pub fn try_spend(&mut self, cost: u32) -> bool {
        if self.marks >= cost {
            self.marks -= cost;
            true
        } else {
            false
        }
    }
}

fn reset(mut treasury: ResMut<Treasury>) {
    *treasury = Treasury::default();
}

/// Runs the moment something sinks. U-boats have no `Bounty`, so the query
/// simply misses and nothing is paid — no branch, no special case.
fn award_bounty(sunk: On<Sunk>, bounties: Query<&Bounty>, mut treasury: ResMut<Treasury>) {
    if let Ok(bounty) = bounties.get(sunk.entity) {
        treasury.marks += bounty.0;
    }
}
