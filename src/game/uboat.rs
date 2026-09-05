//! U-boats: the one mechanic this game has that Kingdom Rush does not.
//!
//! Design rationale, because it drove the whole architecture:
//!
//! A tower defense dies when optimal play is "build the known-best layout, then
//! watch". Towers are a *pre-wave* decision; once the wave starts the player is
//! a spectator. The u-boat pen adds a *mid-wave* decision layer — Kingdom Rush
//! solves the same problem with heroes and spells.
//!
//! The brief said the bunker "creates u-boats that help me defend". Taken
//! literally — free boats spawning on a timer, fighting on their own — that
//! makes the idle problem *worse*: more happens on screen while the player does
//! less. So the same fiction is kept and the agency is put back in:
//!
//! * they cost marks, so they compete with towers for the same budget;
//! * they take time to build, so ordering one is a bet on the next wave;
//! * the player picks the station tile and picks the moment;
//! * they carry four torpedoes, then must sail home to rearm;
//! * destroyers sink them.
//!
//! Same story, but every u-boat is now a decision with a cost and a risk.

use bevy::prelude::*;

use crate::balance::{
    UBOAT_BUILD_SECONDS, UBOAT_COST, UBOAT_HEALTH, UBOAT_PEN_SLOTS, UBOAT_RANGE, UBOAT_SPEED,
    UBOAT_TORPEDOES, UBOAT_TORPEDO_DAMAGE, UBOAT_TORPEDO_RELOAD, Z_UBOAT,
};
use crate::game::combat::{DamageKind, Health};
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::map::RiverPath;
use crate::game::orders::{LaunchUBoat, OrderUBoat};
use crate::game::projectile::projectile_bundle;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UBoatPen>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            FixedUpdate,
            (
                (build_uboats, launch_uboats).chain().in_set(Sim::Spawn),
                navigate.in_set(Sim::Movement),
                fire_torpedoes.in_set(Sim::Targeting),
            ),
        )
        .add_observer(free_slot_on_loss);
}

/// The pen. Durable state, one instance, no relationships: a `Resource`.
#[derive(Resource, Debug)]
pub struct UBoatPen {
    pub slots: u8,
    pub docked: u8,
    pub at_sea: u8,
    pub building: Option<Timer>,
}

impl Default for UBoatPen {
    fn default() -> Self {
        Self {
            slots: UBOAT_PEN_SLOTS,
            docked: 1, // one boat ready at the start, so the mechanic is discoverable
            at_sea: 0,
            building: None,
        }
    }
}

impl UBoatPen {
    pub fn total(&self) -> u8 {
        self.docked + self.at_sea + self.building.is_some() as u8
    }
}

#[derive(Component)]
pub struct UBoat;

#[derive(Component)]
pub struct Torpedoes {
    pub remaining: u8,
    pub tube: Timer,
}

/// Where this boat is on the river and where it wants to be.
///
/// U-boats travel the same `RiverPath` as the enemy, in the opposite
/// direction. One path, two consumers, no second data structure.
#[derive(Component)]
pub struct Nav {
    /// Waypoint currently being steered towards.
    pub index: usize,
    /// Station to hold, as a river index.
    pub station: usize,
    pub returning: bool,
}

fn reset(mut pen: ResMut<UBoatPen>) {
    *pen = UBoatPen::default();
}

fn build_uboats(
    time: Res<Time>,
    mut orders: MessageReader<OrderUBoat>,
    mut pen: ResMut<UBoatPen>,
    mut treasury: ResMut<Treasury>,
) {
    for _ in orders.read() {
        if pen.building.is_some() || pen.total() >= pen.slots {
            continue;
        }
        if treasury.try_spend(UBOAT_COST) {
            pen.building = Some(Timer::from_seconds(UBOAT_BUILD_SECONDS, TimerMode::Once));
        }
    }

    // `take`/`put back` instead of holding a mutable borrow across the check.
    if let Some(mut timer) = pen.building.take() {
        if timer.tick(time.delta()).just_finished() {
            pen.docked += 1;
        } else {
            pen.building = Some(timer);
        }
    }
}

fn launch_uboats(
    mut launches: MessageReader<LaunchUBoat>,
    mut commands: Commands,
    mut pen: ResMut<UBoatPen>,
    path: Res<RiverPath>,
) {
    for launch in launches.read() {
        if pen.docked == 0 {
            continue;
        }
        pen.docked -= 1;
        pen.at_sea += 1;

        let last = path.points.len() - 1;
        commands.spawn((
            Name::new("U-Boot"),
            UBoat,
            Health::new(UBOAT_HEALTH),
            Torpedoes {
                remaining: UBOAT_TORPEDOES,
                tube: Timer::from_seconds(UBOAT_TORPEDO_RELOAD, TimerMode::Repeating),
            },
            Nav {
                // Leaves the bunker heading upstream, i.e. index decreasing.
                index: last.saturating_sub(1),
                station: launch.station.min(last),
                returning: false,
            },
            Sprite {
                color: Color::srgb(0.30, 0.34, 0.38),
                custom_size: Some(Vec2::new(10.0, 26.0)),
                ..default()
            },
            Transform::from_translation(path.bunker_point().extend(Z_UBOAT)),
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

/// Sail to the station, hold there, sail home when the tubes are empty.
///
/// Written as one system rather than three (outbound / holding / returning)
/// because they share every line except which index is the goal. Three systems
/// would need a state component, three queries and an ordering rule — a
/// state machine bolted onto something that is already just "walk to an index".
fn navigate(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    mut pen: ResMut<UBoatPen>,
    mut boats: Query<(Entity, &mut Transform, &mut Nav, &Torpedoes), With<UBoat>>,
) {
    let step = time.delta_secs();
    let last = path.points.len() - 1;

    for (entity, mut transform, mut nav, torpedoes) in &mut boats {
        if torpedoes.remaining == 0 {
            nav.returning = true;
        }

        let goal = if nav.returning { last } else { nav.station };

        // Reached home with empty tubes: dock, free the boat, keep the slot.
        if nav.returning && nav.index >= last {
            pen.at_sea = pen.at_sea.saturating_sub(1);
            pen.docked += 1;
            commands.entity(entity).despawn();
            continue;
        }

        let target = path.points[nav.index];
        let position = transform.translation.truncate();
        let to_target = target - position;
        let travel = UBOAT_SPEED * step;

        if to_target.length() <= travel {
            transform.translation = target.extend(transform.translation.z);
            // Step one waypoint towards the goal, in whichever direction that is.
            if nav.index < goal {
                nav.index += 1;
            } else if nav.index > goal {
                nav.index -= 1;
            }
        } else {
            transform.translation += (to_target.normalize_or_zero() * travel).extend(0.0);
        }

        if to_target.length_squared() > f32::EPSILON {
            transform.rotation = Quat::from_rotation_z(to_target.to_angle() - std::f32::consts::FRAC_PI_2);
        }
    }
}

/// Torpedoes ignore armour — the entire reason a u-boat exists.
fn fire_torpedoes(
    time: Res<Time>,
    mut commands: Commands,
    mut boats: Query<(&Transform, &mut Torpedoes), With<UBoat>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (transform, mut torpedoes) in &mut boats {
        if torpedoes.remaining == 0 || !torpedoes.tube.tick(time.delta()).just_finished() {
            continue;
        }
        let origin = transform.translation.truncate();
        let target = enemies
            .iter()
            .filter(|(_, enemy)| {
                origin.distance_squared(enemy.translation.truncate()) <= UBOAT_RANGE * UBOAT_RANGE
            })
            .min_by(|a, b| {
                origin
                    .distance_squared(a.1.translation.truncate())
                    .total_cmp(&origin.distance_squared(b.1.translation.truncate()))
            });

        if let Some((entity, _)) = target {
            torpedoes.remaining -= 1;
            commands.spawn(projectile_bundle(
                origin,
                entity,
                UBOAT_TORPEDO_DAMAGE,
                DamageKind::Torpedo,
                260.0,
            ));
        }
    }
}

/// A u-boat that sinks does not come back. The slot is freed, the boat is not.
fn free_slot_on_loss(
    sunk: On<crate::game::combat::Sunk>,
    boats: Query<(), With<UBoat>>,
    mut pen: ResMut<UBoatPen>,
) {
    if boats.get(sunk.entity).is_ok() {
        pen.at_sea = pen.at_sea.saturating_sub(1);
    }
}
