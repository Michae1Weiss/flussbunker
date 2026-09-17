//! The player's U-boat: river movement, heading, dive state, torpedoes, deck gun, ramming.
//! Sole writer of `Flotilla`, `Trim`, `Heading`, `Torpedoes`, `Station`, `DeckGun`, `Rammed`.
//! Rules: docs/design/uboat.md

use bevy::prelude::*;

use crate::balance::{
    rank_for, SizeClass, DECK_GUN_DAMAGE, DECK_GUN_RANGE, DECK_GUN_RELOAD, DECK_GUN_SHELL_SPEED,
    RAM_HEAVY_COST, RAM_LIGHT_COST, RAM_SHOVE, TILE_SIZE, TORPEDO_RUN, TORPEDO_SPEED,
    UBOAT_FIRE_ALIGNMENT, UBOAT_HEALTH, UBOAT_LENGTH, UBOAT_MOVE_ALIGNMENT, UBOAT_REARM_SECONDS,
    UBOAT_REPLACEMENT_COST, UBOAT_SALVO_SPACING, UBOAT_SPEED, UBOAT_SUBMERGED_SPEED_FACTOR,
    UBOAT_TRIM_SECONDS, UBOAT_TURN_RATE, Z_UBOAT,
};
use crate::game::combat::{Damage, DamageKind, Health, Sunk, Weapon};
use crate::game::economy::Treasury;
use crate::game::enemy::{Enemy, EnemyType};
use crate::game::input::{CommissionUBoat, FireTorpedo, MoveUBoat, ToggleTrim};
use crate::game::map::RiverPath;
use crate::game::projectile::{shell_bundle, torpedo_bundle};
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
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
                (rearm, fire_torpedoes, fire_deck_gun, collide_with_ships).in_set(Sim::Targeting),
            ),
        )
        .add_observer(on_boat_lost)
        .add_observer(on_kill_credit);
}

/// Outlives the boat entity. `at_sea` and `kills` are reset by `on_boat_lost`,
/// which runs before the entity despawns.
#[derive(Resource, Default, Debug)]
pub struct Flotilla {
    pub kills: u32,
    pub at_sea: bool,
    /// RM for the next boat.
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

/// Dive state. A toggle during a transition reverses it, keeping the elapsed depth.
#[derive(Component, Debug)]
pub enum Trim {
    Surfaced,
    Submerging(Timer),
    Submerged,
    Surfacing(Timer),
}

impl Trim {
    pub fn is_hittable(&self) -> bool {
        !matches!(self, Trim::Submerged)
    }

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

    fn toggled(&self) -> Self {
        let fresh = || Timer::from_seconds(UBOAT_TRIM_SECONDS, TimerMode::Once);
        let reversed = |t: &Timer| {
            let mut timer = fresh();
            timer.set_elapsed(t.remaining());
            timer
        };
        match self {
            Trim::Surfaced => Trim::Submerging(fresh()),
            Trim::Submerged => Trim::Surfacing(fresh()),
            Trim::Submerging(t) => Trim::Surfacing(reversed(t)),
            Trim::Surfacing(t) => Trim::Submerging(reversed(t)),
        }
    }
}

/// Unit bow direction. Follows the direction of travel; unchanged at rest.
#[derive(Component)]
pub struct Heading(pub Vec2);

/// Auto-fires at the nearest enemy in range while surfaced.
#[derive(Component)]
pub struct DeckGun {
    /// Ready when finished; reset after each shot.
    pub reload: Timer,
}

/// An enemy that has already collided with the boat. A ship rams once.
#[derive(Component)]
pub struct Rammed;

#[derive(Component)]
pub struct Torpedoes {
    pub loaded: u8,
    /// Spacing between tubes of a salvo; a torpedo fires only once this has finished.
    pub tube: Timer,
    /// Per-torpedo resupply while docked.
    pub rearm: Timer,
}

/// Position and destination as arc lengths along `RiverPath`.
#[derive(Component)]
pub struct Station {
    pub along: f32,
    pub goal: f32,
}

impl Station {
    /// Within a tile of the bunker.
    fn is_docked(&self, path: &RiverPath) -> bool {
        self.along >= path.length() - TILE_SIZE
    }
}

fn commission_first_boat(mut intents: MessageWriter<CommissionUBoat>) {
    intents.write(CommissionUBoat { free: true });
}

fn commission(
    mut intents: MessageReader<CommissionUBoat>,
    mut commands: Commands,
    mut flotilla: ResMut<Flotilla>,
    mut treasury: ResMut<Treasury>,
    path: Res<RiverPath>,
) {
    for intent in intents.read() {
        if flotilla.at_sea {
            continue;
        }
        if !intent.free && !treasury.try_spend(flotilla.replacement_cost) {
            continue;
        }

        let rank = rank_for(flotilla.kills);
        flotilla.at_sea = true;
        let mut tube = Timer::from_seconds(UBOAT_SALVO_SPACING, TimerMode::Once);
        tube.set_elapsed(tube.duration());
        let mut gun = Timer::from_seconds(DECK_GUN_RELOAD, TimerMode::Once);
        gun.set_elapsed(gun.duration());

        commands.spawn((
            Name::new("U-96"),
            UBoat,
            Health::new(UBOAT_HEALTH),
            Trim::Surfaced,
            Heading(-path.tangent_at(path.length())),
            DeckGun { reload: gun },
            Torpedoes {
                loaded: rank.torpedoes,
                tube,
                rearm: Timer::from_seconds(UBOAT_REARM_SECONDS, TimerMode::Repeating),
            },
            Station {
                along: path.length(),
                goal: path.length(),
            },
            Sprite {
                color: Color::srgb(0.34, 0.38, 0.42),
                custom_size: Some(Vec2::new(12.0, UBOAT_LENGTH)),
                ..default()
            },
            Transform::from_translation(path.bunker_point().extend(Z_UBOAT)),
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

fn apply_move_intents(
    mut intents: MessageReader<MoveUBoat>,
    mut boats: Query<&mut Station, With<UBoat>>,
) {
    for intent in intents.read() {
        for mut station in &mut boats {
            station.goal = intent.along;
        }
    }
}

/// Surfacing is refused while a medium or heavier ship is directly above.
fn apply_trim_intents(
    mut intents: MessageReader<ToggleTrim>,
    mut boats: Query<(&Transform, &mut Trim), With<UBoat>>,
    ships: Query<(&Transform, &EnemyType), With<Enemy>>,
) {
    for _ in intents.read() {
        for (transform, mut trim) in &mut boats {
            let surfacing = matches!(*trim, Trim::Submerged | Trim::Submerging(_));
            let blocked = ships.iter().any(|(ship, kind)| {
                kind.0.stats().class != SizeClass::Light && overlaps(transform, ship, kind)
            });
            if surfacing && blocked {
                continue;
            }
            *trim = trim.toggled();
        }
    }
}

fn overlaps(boat: &Transform, ship: &Transform, kind: &EnemyType) -> bool {
    let reach = (kind.0.stats().size + UBOAT_LENGTH) * 0.5;
    boat.translation
        .truncate()
        .distance_squared(ship.translation.truncate())
        <= reach * reach
}

fn navigate(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut boats: Query<
        (
            &mut Transform,
            &mut Station,
            &mut Heading,
            &mut Trim,
            &mut Sprite,
        ),
        With<UBoat>,
    >,
) {
    let step = time.delta_secs();

    for (mut transform, mut station, mut heading, mut trim, mut sprite) in &mut boats {
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

        sprite.color = match *trim {
            Trim::Surfaced => Color::srgb(0.42, 0.47, 0.52),
            Trim::Submerged => Color::srgb(0.16, 0.22, 0.30),
            _ => Color::srgb(0.28, 0.34, 0.42),
        };

        let speed = match *trim {
            Trim::Surfaced => UBOAT_SPEED,
            _ => UBOAT_SPEED * UBOAT_SUBMERGED_SPEED_FACTOR,
        };

        // Stop, turn on the spot, then move: the tangent is sampled slightly ahead so a
        // boat standing exactly on a bend turns toward the straight it is about to enter.
        let remaining = station.goal - station.along;
        if remaining != 0.0 {
            let ahead = station.along + remaining.signum();
            let wanted = path.tangent_at(ahead) * remaining.signum();
            heading.0 = heading.0.rotate_towards(wanted, UBOAT_TURN_RATE * step);
            if heading.0.angle_to(wanted).abs() <= UBOAT_MOVE_ALIGNMENT {
                let travel = (speed * step).min(remaining.abs());
                station.along += travel.copysign(remaining);
            }
        }

        let z = transform.translation.z;
        transform.translation = path.point_at(station.along).extend(z);
        transform.rotation =
            Quat::from_rotation_z(heading.0.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

fn rearm(
    time: Res<Time>,
    path: Res<RiverPath>,
    flotilla: Res<Flotilla>,
    mut boats: Query<(&Station, &mut Torpedoes), With<UBoat>>,
) {
    for (station, mut torpedoes) in &mut boats {
        let capacity = rank_for(flotilla.kills).torpedoes;
        if !station.is_docked(&path) || torpedoes.loaded >= capacity {
            torpedoes.rearm.reset();
            continue;
        }
        if torpedoes.rearm.tick(time.delta()).just_finished() {
            torpedoes.loaded += 1;
        }
    }
}

/// One torpedo per `FireTorpedo` intent, along the river axis the bow is aligned
/// with. Refused while submerged, turning, or with empty tubes.
fn fire_torpedoes(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    flotilla: Res<Flotilla>,
    mut intents: MessageReader<FireTorpedo>,
    mut boats: Query<(&Transform, &Station, &Heading, &Trim, &mut Torpedoes), With<UBoat>>,
) {
    let rank = rank_for(flotilla.kills);
    let requested = intents.read().count() > 0;

    for (transform, station, heading, trim, mut torpedoes) in &mut boats {
        torpedoes.tube.tick(time.delta());
        if !requested || !trim.can_fire() || torpedoes.loaded == 0 || !torpedoes.tube.is_finished()
        {
            continue;
        }

        let tangent = path.tangent_at(station.along);
        let axis = if heading.0.dot(tangent) >= 0.0 {
            tangent
        } else {
            -tangent
        };
        if heading.0.angle_to(axis).abs() > UBOAT_FIRE_ALIGNMENT {
            continue;
        }

        torpedoes.loaded -= 1;
        torpedoes.tube.reset();
        commands.spawn(torpedo_bundle(
            transform.translation.truncate(),
            axis,
            rank.damage,
            TORPEDO_SPEED,
            TORPEDO_RUN,
        ));
    }
}

fn fire_deck_gun(
    time: Res<Time>,
    mut commands: Commands,
    mut boats: Query<(&Transform, &Trim, &mut DeckGun), With<UBoat>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (transform, trim, mut gun) in &mut boats {
        gun.reload.tick(time.delta());
        if !trim.can_fire() || !gun.reload.is_finished() {
            continue;
        }
        let origin = transform.translation.truncate();
        let target = enemies
            .iter()
            .map(|(entity, ship)| (entity, origin.distance_squared(ship.translation.truncate())))
            .filter(|(_, distance)| *distance <= DECK_GUN_RANGE * DECK_GUN_RANGE)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((entity, _)) = target else {
            continue;
        };
        gun.reload.reset();
        commands.spawn(shell_bundle(
            origin,
            entity,
            DECK_GUN_DAMAGE,
            false,
            DECK_GUN_SHELL_SPEED,
            Weapon::DeckGun,
        ));
    }
}

/// Ships never stop: a light boat capsizes on a surfaced U-boat, a heavier ship
/// damages and shoves it downstream. Each ship collides at most once.
fn collide_with_ships(
    mut commands: Commands,
    path: Res<RiverPath>,
    mut damage: MessageWriter<Damage>,
    mut boats: Query<(Entity, &Transform, &Trim, &Health, &mut Station), With<UBoat>>,
    ships: Query<(Entity, &Transform, &EnemyType, &Health), (With<Enemy>, Without<Rammed>)>,
) {
    for (boat, transform, trim, health, mut station) in &mut boats {
        if !trim.is_hittable() {
            continue;
        }
        for (ship, ship_transform, kind, ship_health) in &ships {
            if !overlaps(transform, ship_transform, kind) {
                continue;
            }
            commands.entity(ship).insert(Rammed);
            let cost = match kind.0.stats().class {
                SizeClass::Light => {
                    damage.write(Damage {
                        target: ship,
                        amount: ship_health.max,
                        kind: DamageKind::Ram,
                        ap: false,
                        weapon: Weapon::Ram,
                    });
                    RAM_LIGHT_COST
                }
                _ => {
                    let at_rest = station.goal == station.along;
                    station.along = (station.along + RAM_SHOVE).min(path.length());
                    if at_rest {
                        station.goal = station.along;
                    }
                    RAM_HEAVY_COST
                }
            };
            damage.write(Damage {
                target: boat,
                amount: health.max * cost,
                kind: DamageKind::Ram,
                ap: false,
                weapon: Weapon::Ram,
            });
        }
    }
}

fn on_kill_credit(
    sunk: On<Sunk>,
    enemies: Query<&crate::game::combat::LastHitBy>,
    mut flotilla: ResMut<Flotilla>,
) {
    if let Ok(last) = enemies.get(sunk.entity) {
        if last.0 == Weapon::Torpedo {
            flotilla.kills += 1;
        }
    }
}

fn on_boat_lost(sunk: On<Sunk>, boats: Query<(), With<UBoat>>, mut flotilla: ResMut<Flotilla>) {
    if boats.get(sunk.entity).is_ok() {
        flotilla.at_sea = false;
        flotilla.kills = 0;
    }
}
