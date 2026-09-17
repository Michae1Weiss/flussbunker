//! The player's U-boat: models, river movement, heading, dive state, torpedoes, deck gun,
//! ramming, loss and return. Sole writer of `Flotilla`, `Model`, `Trim`, `Heading`,
//! `Torpedoes`, `Station`, `DeckGun`, `Rammed`. Rules: docs/design/uboat.md

use bevy::prelude::*;

use crate::balance::{
    BoatModel, SizeClass, DECK_GUN_DAMAGE, DECK_GUN_RANGE, DECK_GUN_RELOAD, DECK_GUN_SHELL_SPEED,
    DOCK_REPAIR_PER_SECOND, RAM_HEAVY_COST, RAM_LIGHT_COST, RAM_SHOVE, REPAIR_BILL, TILE_SIZE,
    TORPEDO_RUN, TORPEDO_SPEED, UBOAT_FIRE_ALIGNMENT, UBOAT_LENGTH, UBOAT_MOVE_ALIGNMENT,
    UBOAT_REARM_SECONDS, UBOAT_SALVO_SPACING, UBOAT_SUBMERGED_SPEED_FACTOR, UBOAT_TURN_RATE,
    UNPAID_RETURN_HP, Z_UBOAT,
};
use crate::game::combat::{Damage, DamageKind, Health, LastHitBy, Sunk, Weapon};
use crate::game::economy::Treasury;
use crate::game::enemy::{Enemy, EnemyType};
use crate::game::input::{FireTorpedo, MoveUBoat, ToggleTrim, UpgradeBoat};
use crate::game::map::RiverPath;
use crate::game::notes::{Hint, HintKey};
use crate::game::projectile::{shell_bundle, torpedo_bundle};
use crate::game::wave::WaveFinished;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Flotilla>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            FixedUpdate,
            (
                (
                    return_after_wave,
                    apply_upgrades,
                    apply_move_intents,
                    apply_trim_intents,
                )
                    .chain()
                    .in_set(Sim::Spawn),
                navigate.in_set(Sim::Movement),
                (
                    dock,
                    fire_torpedoes,
                    fire_deck_gun,
                    collide_with_ships,
                    shelled,
                )
                    .in_set(Sim::Targeting),
            ),
        )
        .add_observer(on_boat_lost);
}

/// Outlives the boat entity. `at_sea` is cleared by `on_boat_lost`, which runs before
/// the entity despawns.
#[derive(Resource, Debug)]
pub struct Flotilla {
    pub model: BoatModel,
    pub at_sea: bool,
}

impl Default for Flotilla {
    fn default() -> Self {
        Self {
            model: BoatModel::ViiA,
            at_sea: false,
        }
    }
}

fn reset(mut commands: Commands, mut flotilla: ResMut<Flotilla>, path: Res<RiverPath>) {
    *flotilla = Flotilla::default();
    commission(&mut commands, &mut flotilla, &path, 1.0);
}

#[derive(Component)]
pub struct UBoat;

#[derive(Component, Clone, Copy, Debug)]
pub struct Model(pub BoatModel);

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

    fn toggled(&self, dive_seconds: f32) -> Self {
        let fresh = || Timer::from_seconds(dive_seconds, TimerMode::Once);
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
    pub fn is_docked(&self, path: &RiverPath) -> bool {
        self.along >= path.length() - TILE_SIZE
    }
}

fn ready(seconds: f32) -> Timer {
    let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
    timer.set_elapsed(timer.duration());
    timer
}

/// Greybox size per model; the image, if any, belongs to `art` and is kept.
fn dress(sprite: &mut Sprite, model: BoatModel) {
    let length = UBOAT_LENGTH + 4.0 * model as u8 as f32;
    sprite.custom_size = Some(Vec2::new(12.0, length));
}

/// Spawns the flotilla's model at the bunker with `hp_fraction` of its hull.
fn commission(
    commands: &mut Commands,
    flotilla: &mut Flotilla,
    path: &RiverPath,
    hp_fraction: f32,
) {
    let stats = flotilla.model.stats();
    flotilla.at_sea = true;
    let mut health = Health::new(stats.hp);
    health.current = stats.hp * hp_fraction;
    commands.spawn((
        Name::new("U-96"),
        UBoat,
        Model(flotilla.model),
        health,
        Trim::Surfaced,
        Heading(-path.tangent_at(path.length())),
        DeckGun {
            reload: ready(DECK_GUN_RELOAD),
        },
        Torpedoes {
            loaded: stats.tubes,
            tube: ready(UBOAT_SALVO_SPACING),
            rearm: Timer::from_seconds(UBOAT_REARM_SECONDS, TimerMode::Repeating),
        },
        Station {
            along: path.length(),
            goal: path.length(),
        },
        {
            let mut sprite = Sprite::default();
            dress(&mut sprite, flotilla.model);
            sprite
        },
        Transform::from_translation(path.bunker_point().extend(Z_UBOAT)),
        DespawnOnExit(Screen::Gameplay),
    ));
}

/// A sunk boat is gone for the wave and returns at the next prep phase: full hull if
/// the shipyard bill is paid, `UNPAID_RETURN_HP` otherwise.
fn return_after_wave(
    mut finished: MessageReader<WaveFinished>,
    mut commands: Commands,
    mut flotilla: ResMut<Flotilla>,
    mut treasury: ResMut<Treasury>,
    path: Res<RiverPath>,
) {
    if finished.read().count() == 0 || flotilla.at_sea {
        return;
    }
    let bill = (flotilla.model.stats().nominal as f32 * REPAIR_BILL).round() as u32;
    let fraction = if treasury.try_spend(bill) {
        1.0
    } else {
        UNPAID_RETURN_HP
    };
    commission(&mut commands, &mut flotilla, &path, fraction);
}

/// Buys the next model while docked: full hull, full tubes, new behaviour.
fn apply_upgrades(
    mut intents: MessageReader<UpgradeBoat>,
    mut flotilla: ResMut<Flotilla>,
    mut treasury: ResMut<Treasury>,
    path: Res<RiverPath>,
    mut boats: Query<
        (
            &Station,
            &mut Model,
            &mut Health,
            &mut Torpedoes,
            &mut Sprite,
        ),
        With<UBoat>,
    >,
) {
    for _ in intents.read() {
        let Ok((station, mut model, mut health, mut torpedoes, mut sprite)) = boats.single_mut()
        else {
            continue;
        };
        let Some(next) = flotilla.model.next() else {
            continue;
        };
        if !station.is_docked(&path) || !treasury.try_spend(next.stats().price) {
            continue;
        }
        flotilla.model = next;
        model.0 = next;
        *health = Health::new(next.stats().hp);
        torpedoes.loaded = next.stats().tubes;
        dress(&mut sprite, next);
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
    mut boats: Query<(&Transform, &Model, &mut Trim), With<UBoat>>,
    ships: Query<(&Transform, &EnemyType), With<Enemy>>,
    mut hints: MessageWriter<Hint>,
) {
    for _ in intents.read() {
        for (transform, model, mut trim) in &mut boats {
            let surfacing = matches!(*trim, Trim::Submerged | Trim::Submerging(_));
            let blocked = ships.iter().any(|(ship, kind)| {
                kind.0.stats().class != SizeClass::Light && overlaps(transform, ship, kind)
            });
            if surfacing && blocked {
                hints.write(Hint {
                    key: HintKey::ContactAbove,
                    at: transform.translation.truncate(),
                });
                continue;
            }
            *trim = trim.toggled(model.0.stats().dive_seconds);
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
            &Model,
            &mut Station,
            &mut Heading,
            &mut Trim,
            &mut Sprite,
        ),
        With<UBoat>,
    >,
) {
    let step = time.delta_secs();

    for (mut transform, model, mut station, mut heading, mut trim, mut sprite) in &mut boats {
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

        // A drawing is tinted, a rectangle is coloured.
        let drawn = sprite.image != Handle::default();
        sprite.color = match (&*trim, drawn) {
            (Trim::Surfaced, true) => Color::WHITE,
            (Trim::Submerged, true) => Color::srgb(0.35, 0.45, 0.6),
            (_, true) => Color::srgb(0.65, 0.7, 0.8),
            (Trim::Surfaced, false) => Color::srgb(0.42, 0.47, 0.52),
            (Trim::Submerged, false) => Color::srgb(0.16, 0.22, 0.30),
            (_, false) => Color::srgb(0.28, 0.34, 0.42),
        };

        let speed = match *trim {
            Trim::Surfaced => model.0.stats().speed,
            _ => model.0.stats().speed * UBOAT_SUBMERGED_SPEED_FACTOR,
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

/// Docked: torpedoes resupply one per `UBOAT_REARM_SECONDS`, the hull repairs for free.
fn dock(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut boats: Query<(&Station, &Model, &mut Torpedoes, &mut Health), With<UBoat>>,
) {
    for (station, model, mut torpedoes, mut health) in &mut boats {
        if !station.is_docked(&path) {
            torpedoes.rearm.reset();
            continue;
        }
        health.current = (health.current + health.max * DOCK_REPAIR_PER_SECOND * time.delta_secs())
            .min(health.max);
        if torpedoes.loaded >= model.0.stats().tubes {
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
    mut intents: MessageReader<FireTorpedo>,
    mut hints: MessageWriter<Hint>,
    mut boats: Query<
        (
            &Transform,
            &Model,
            &Station,
            &Heading,
            &Trim,
            &mut Torpedoes,
        ),
        With<UBoat>,
    >,
) {
    let requested = intents.read().count() > 0;

    for (transform, model, station, heading, trim, mut torpedoes) in &mut boats {
        torpedoes.tube.tick(time.delta());
        let at = transform.translation.truncate();
        if !requested || !trim.can_fire() || !torpedoes.tube.is_finished() {
            continue;
        }
        if torpedoes.loaded == 0 {
            hints.write(Hint {
                key: HintKey::EmptyTubes,
                at,
            });
            continue;
        }

        let tangent = path.tangent_at(station.along);
        let axis = if heading.0.dot(tangent) >= 0.0 {
            tangent
        } else {
            -tangent
        };
        if heading.0.angle_to(axis).abs() > UBOAT_FIRE_ALIGNMENT {
            hints.write(Hint {
                key: HintKey::BowNotAligned,
                at,
            });
            continue;
        }

        torpedoes.loaded -= 1;
        torpedoes.tube.reset();
        commands.spawn(torpedo_bundle(
            transform.translation.truncate(),
            axis,
            model.0.stats().torpedo_damage,
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

/// Two destroyer hits without diving earn the dive hint.
fn shelled(
    mut hints: MessageWriter<Hint>,
    mut hits: Local<u32>,
    boats: Query<(&Transform, &LastHitBy, &Trim), (With<UBoat>, Changed<LastHitBy>)>,
) {
    for (transform, last, trim) in &boats {
        if last.0 != Weapon::Enemy || !matches!(trim, Trim::Surfaced) {
            continue;
        }
        *hits += 1;
        if *hits >= 2 {
            hints.write(Hint {
                key: HintKey::Dive,
                at: transform.translation.truncate(),
            });
        }
    }
}

fn on_boat_lost(sunk: On<Sunk>, boats: Query<(), With<UBoat>>, mut flotilla: ResMut<Flotilla>) {
    if boats.get(sunk.entity).is_ok() {
        flotilla.at_sea = false;
    }
}
