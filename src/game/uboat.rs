//! U-96: the boat you command.
//!
//! ## What the playtest changed, and why
//!
//! v1 made the u-boat a *purchase*: press U, wait 9 s invisibly, watch a
//! counter, click water, wait 12 s more for it to arrive, get four homing shots
//! at 2.4 tiles. Three findings killed it:
//!
//! * "Minen platzieren habe ich gedacht" — clicking water was bound to the
//!   wrong verb. The hand expected *something happens at that tile*.
//! * "I love torpedos" / best moment was an impact — the payoff is the hit, and
//!   ~21 s of logistics bought ~4 of them.
//! * Three rounds played without discovering that a second boat was possible —
//!   a mechanic behind a keypress and a text counter does not exist.
//!
//! v2 therefore makes it a **persistent unit**, present from the first second:
//!
//! * No build step. You have U-96 from wave 1. Click water, it goes there.
//! * **Torpedoes run straight down the channel.** They are not homing and have
//!   no radius: they travel the water until they hit a hull or lose the river.
//!   This one change does four jobs at once — firing over land becomes
//!   structurally impossible, range can be generous without being boring, a
//!   torpedo down a packed straight can hit several boats, and *the shape of
//!   the river becomes the tactical map*. Long straights are firing lanes,
//!   bends are dead water. The tile you pick finally matters.
//! * **Dive** (`T`) makes you untouchable and blind. The cost is the 1.4 s trim
//!   time — without it, permanently submerged would be free and the decision
//!   would not exist.
//! * **Veterancy** across waves, and losing the boat loses the rank. Permanent
//!   progression is what makes a unit worth protecting.
//!
//! Note what is *not* here: no pen, no slots, no build timer, no second boat.
//! Roughly 60 lines of v1 bookkeeping deleted, and the mechanic got better. The
//! cheapest complexity to remove is the kind that was never fun.

use bevy::prelude::*;

use crate::balance::{
    rank_for, TORPEDO_RUN, TORPEDO_SPEED, UBOAT_HEALTH, UBOAT_REARM_SECONDS,
    UBOAT_REPLACEMENT_COST, UBOAT_SPEED, UBOAT_SUBMERGED_SPEED_FACTOR, UBOAT_TRIM_SECONDS, Z_UBOAT,
};
use crate::game::combat::{DamageKind, Health, Sunk};
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::map::RiverPath;
use crate::game::input::{CommissionUBoat, MoveUBoat, ToggleTrim};
use crate::game::projectile::torpedo_bundle;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Flotilla>()
        .add_systems(
            OnEnter(Screen::Gameplay),
            (reset, commission_first_boat).chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                (commission, apply_move_intents, apply_trim_intents).in_set(Sim::Spawn),
                navigate.in_set(Sim::Movement),
                (rearm, fire_torpedoes).in_set(Sim::Targeting),
            ),
        )
        .add_observer(on_boat_lost)
        .add_observer(on_kill_credit);
}

/// What survives the boat: the tally, and whether one is currently at sea.
///
/// Kills live here rather than on the entity precisely *because* the entity is
/// destructible — and the rule is that a sinking wipes the record. Storing it
/// on the boat would have made that rule automatic; storing it here makes it
/// explicit, which is better, because "veterancy resets" is a design promise
/// and design promises should be visible in the code that enforces them.
#[derive(Resource, Default, Debug)]
pub struct Flotilla {
    pub kills: u32,
    pub at_sea: bool,
    /// Cost of the next boat. Free the first time.
    pub replacement_cost: u32,
}

fn reset(mut flotilla: ResMut<Flotilla>) {
    *flotilla = Flotilla {
        kills: 0,
        at_sea: false,
        replacement_cost: UBOAT_REPLACEMENT_COST,
    };
}

#[derive(Component)]
pub struct UBoat;

/// Dive state. A four-variant machine is worth spelling out here: the two
/// transition states are not decoration, they *are* the cost of diving, and a
/// bool plus a timer would hide that.
#[derive(Component, Debug)]
pub enum Trim {
    Surfaced,
    Submerging(Timer),
    Submerged,
    Surfacing(Timer),
}

impl Trim {
    /// Only a fully submerged boat is safe from depth charges.
    pub fn is_hittable(&self) -> bool {
        !matches!(self, Trim::Submerged)
    }

    /// Only a fully surfaced boat can shoot. Diving to hide means giving up the
    /// thing you came for — that tension is the mechanic.
    pub fn can_fire(&self) -> bool {
        matches!(self, Trim::Surfaced)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Trim::Surfaced => "aufgetaucht",
            Trim::Submerging(_) => "taucht...",
            Trim::Submerged => "getaucht",
            Trim::Surfacing(_) => "taucht auf...",
        }
    }
}

#[derive(Component)]
pub struct Torpedoes {
    pub loaded: u8,
    pub tube: Timer,
    pub rearm: Timer,
}

/// Position along the river, and where it has been ordered to.
#[derive(Component)]
pub struct Station {
    /// Waypoint currently being steered towards.
    pub index: usize,
    /// Ordered destination.
    pub goal: usize,
}

fn commission_first_boat(mut orders: MessageWriter<CommissionUBoat>) {
    // You start with a boat. Not a menu, not a counter, not a purchase — the
    // unit you command is on screen before the first enemy is.
    orders.write(CommissionUBoat { free: true });
}

fn commission(
    mut orders: MessageReader<CommissionUBoat>,
    mut commands: Commands,
    mut flotilla: ResMut<Flotilla>,
    mut treasury: ResMut<Treasury>,
    path: Res<RiverPath>,
) {
    for order in orders.read() {
        if flotilla.at_sea {
            continue;
        }
        if !order.free && !treasury.try_spend(flotilla.replacement_cost) {
            continue;
        }

        let dock = path.points.len() - 1;
        let rank = rank_for(flotilla.kills);
        flotilla.at_sea = true;

        commands.spawn((
            Name::new("U-96"),
            UBoat,
            Health::new(UBOAT_HEALTH),
            Trim::Surfaced,
            Torpedoes {
                loaded: rank.torpedoes,
                tube: Timer::from_seconds(rank.reload, TimerMode::Repeating),
                rearm: Timer::from_seconds(UBOAT_REARM_SECONDS, TimerMode::Repeating),
            },
            Station {
                index: dock,
                goal: dock,
            },
            Sprite {
                color: Color::srgb(0.34, 0.38, 0.42),
                custom_size: Some(Vec2::new(12.0, 30.0)),
                ..default()
            },
            Transform::from_translation(path.bunker_point().extend(Z_UBOAT)),
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

fn apply_move_intents(
    mut orders: MessageReader<MoveUBoat>,
    mut boats: Query<&mut Station, With<UBoat>>,
) {
    for order in orders.read() {
        for mut station in &mut boats {
            station.goal = order.station;
        }
    }
}

fn apply_trim_intents(
    mut orders: MessageReader<ToggleTrim>,
    mut boats: Query<&mut Trim, With<UBoat>>,
) {
    for _ in orders.read() {
        for mut trim in &mut boats {
            // Orders during a transition are ignored rather than queued. A
            // queued dive that fires 1.4 s after you stopped wanting it is
            // worse than no response at all.
            *trim = match *trim {
                Trim::Surfaced => {
                    Trim::Submerging(Timer::from_seconds(UBOAT_TRIM_SECONDS, TimerMode::Once))
                }
                Trim::Submerged => {
                    Trim::Surfacing(Timer::from_seconds(UBOAT_TRIM_SECONDS, TimerMode::Once))
                }
                Trim::Submerging(ref t) => Trim::Submerging(t.clone()),
                Trim::Surfacing(ref t) => Trim::Surfacing(t.clone()),
            };
        }
    }
}

fn navigate(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut boats: Query<(&mut Transform, &mut Station, &mut Trim, &mut Sprite), With<UBoat>>,
) {
    let step = time.delta_secs();

    for (mut transform, mut station, mut trim, mut sprite) in &mut boats {
        // Advance the dive state machine.
        match &mut *trim {
            Trim::Submerging(t) => {
                if t.tick(time.delta()).just_finished() {
                    *trim = Trim::Submerged;
                }
            }
            Trim::Surfacing(t) => {
                if t.tick(time.delta()).just_finished() {
                    *trim = Trim::Surfaced;
                }
            }
            Trim::Surfaced | Trim::Submerged => {}
        }

        // Depth is readable at a glance instead of via the HUD: a submerged
        // boat is dim. One line of code, and the state is never a surprise.
        sprite.color = match *trim {
            Trim::Surfaced => Color::srgb(0.42, 0.47, 0.52),
            Trim::Submerged => Color::srgb(0.16, 0.22, 0.30),
            _ => Color::srgb(0.28, 0.34, 0.42),
        };

        let speed = match *trim {
            Trim::Surfaced => UBOAT_SPEED,
            _ => UBOAT_SPEED * UBOAT_SUBMERGED_SPEED_FACTOR,
        };

        let target = path.points[station.index];
        let position = transform.translation.truncate();
        let to_target = target - position;
        let travel = speed * step;

        if to_target.length() <= travel {
            // Standing on the current waypoint. Only *now* may the index step
            // towards the goal — advancing it before the leg is sailed is what
            // makes a boat park one tile short of the tile you clicked.
            transform.translation = target.extend(transform.translation.z);
            if station.index < station.goal {
                station.index += 1;
            } else if station.index > station.goal {
                station.index -= 1;
            } else {
                continue;
            }
        } else {
            transform.translation += (to_target.normalize_or_zero() * travel).extend(0.0);
        }

        if to_target.length_squared() > f32::EPSILON {
            transform.rotation =
                Quat::from_rotation_z(to_target.to_angle() - std::f32::consts::FRAC_PI_2);
        }
    }
}

/// Reloading happens only at the bunker. That is the rhythm of the mechanic:
/// sortie, spend, run home, sortie again — and deciding *when* to break off is
/// the interesting part.
fn rearm(
    time: Res<Time>,
    path: Res<RiverPath>,
    flotilla: Res<Flotilla>,
    mut boats: Query<(&Station, &mut Torpedoes), With<UBoat>>,
) {
    let dock = path.points.len() - 1;
    for (station, mut torpedoes) in &mut boats {
        let capacity = rank_for(flotilla.kills).torpedoes;
        if station.index < dock.saturating_sub(1) || torpedoes.loaded >= capacity {
            torpedoes.rearm.reset();
            continue;
        }
        if torpedoes.rearm.tick(time.delta()).just_finished() {
            torpedoes.loaded += 1;
        }
    }
}

/// Fire straight up the channel.
///
/// The direction is taken from the river itself — the vector from this
/// waypoint to the one upstream. So a boat parked mid-straight has a lane the
/// length of that straight, and a boat sitting on a bend has almost none. No
/// aiming UI, no target selection: **position is the aim.**
fn fire_torpedoes(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    flotilla: Res<Flotilla>,
    mut boats: Query<(&Transform, &Station, &Trim, &mut Torpedoes), With<UBoat>>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    let rank = rank_for(flotilla.kills);

    for (transform, station, trim, mut torpedoes) in &mut boats {
        if !trim.can_fire() || torpedoes.loaded == 0 {
            torpedoes.tube.reset();
            continue;
        }
        if !torpedoes.tube.tick(time.delta()).just_finished() {
            continue;
        }

        // Upstream is index - 1. At the very top of the river there is nothing
        // to shoot at anyway.
        let Some(upstream) = station.index.checked_sub(1) else {
            continue;
        };
        let here = path.points[station.index];
        let heading = (path.points[upstream] - here).normalize_or_zero();
        if heading == Vec2::ZERO {
            continue;
        }

        // Only fire if something is actually in the lane — otherwise a docked
        // boat would empty its tubes into open water. "In the lane" is a cheap
        // projection test: ahead of us, and within half a tile of the axis.
        let origin = transform.translation.truncate();
        let has_target = enemies.iter().any(|enemy| {
            let offset = enemy.translation.truncate() - origin;
            let along = offset.dot(heading);
            let across = (offset - heading * along).length();
            along > 0.0 && along <= TORPEDO_RUN && across <= crate::balance::TILE_SIZE * 0.5
        });
        if !has_target {
            torpedoes.tube.reset();
            continue;
        }

        torpedoes.loaded -= 1;
        commands.spawn(torpedo_bundle(
            origin,
            heading,
            rank.damage,
            TORPEDO_SPEED,
            TORPEDO_RUN,
        ));
    }
}

/// Kill credit. Only torpedo kills count towards rank — the boat's record, not
/// the shore batteries'.
fn on_kill_credit(
    sunk: On<Sunk>,
    enemies: Query<&crate::game::combat::LastHitBy>,
    mut flotilla: ResMut<Flotilla>,
) {
    if let Ok(last) = enemies.get(sunk.entity) {
        if last.0 == DamageKind::Torpedo {
            flotilla.kills += 1;
        }
    }
}

fn on_boat_lost(sunk: On<Sunk>, boats: Query<(), With<UBoat>>, mut flotilla: ResMut<Flotilla>) {
    if boats.get(sunk.entity).is_ok() {
        flotilla.at_sea = false;
        // Harsh and real, as ordered: the record dies with the crew.
        flotilla.kills = 0;
    }
}
