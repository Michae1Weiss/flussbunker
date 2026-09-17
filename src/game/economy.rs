//! Reichsmark. Sole writer of `Treasury`; pays `Bounty` on `Sunk`.

use bevy::prelude::*;

use crate::balance::STARTING_MARKS;
use crate::game::combat::Sunk;
use crate::game::enemy::Bounty;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Treasury>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_observer(award_bounty);
}

#[derive(Resource, Debug)]
pub struct Treasury {
    pub marks: u32,
}

impl Default for Treasury {
    fn default() -> Self {
        Self {
            marks: STARTING_MARKS,
        }
    }
}

impl Treasury {
    /// Deducts `cost` and returns `true`, or leaves the balance untouched and returns `false`.
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

fn award_bounty(sunk: On<Sunk>, bounties: Query<&Bounty>, mut treasury: ResMut<Treasury>) {
    if let Ok(bounty) = bounties.get(sunk.entity) {
        treasury.marks += bounty.0;
    }
}
