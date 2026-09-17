//! The player's U-boat: river movement, heading, dive state, torpedoes, rearming.
//! Sole writer of `Flotilla`, `Trim`, `Heading`, `Torpedoes`, `Station`.
//! Rules: docs/design/uboat.md

use bevy::prelude::*;

use crate::balance::{
    rank_for, TILE_SIZE, TORPEDO_RUN, TORPEDO_SPEED, UBOAT_FIRE_ALIGNMENT, UBOAT_HEALTH,
    UBOAT_REARM_SECONDS, UBOAT_REPLACEMENT_COST, UBOAT_SALVO_SPACING, UBOAT_SPEED,
    UBOAT_SUBMERGED_SPEED_FACTOR, UBOAT_TRIM_SECONDS, UBOAT_TURN_RATE, Z_UBOAT,
};
use crate::game::combat::{DamageKind, Health, Sunk};
use crate::game::economy::Treasury;
use crate::game::input::{CommissionUBoat, FireTorpedo, MoveUBoat, ToggleTrim};
use crate::game::map::RiverPath;
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

        commands.spawn((
            Name::new("U-96"),
            UBoat,
            Health::new(UBOAT_HEALTH),
            Trim::Surfaced,
            Heading(-path.tangent_at(path.length())),
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
                custom_size: Some(Vec2::new(12.0, 30.0)),
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

fn apply_trim_intents(
    mut intents: MessageReader<ToggleTrim>,
    mut boats: Query<&mut Trim, With<UBoat>>,
) {
    for _ in intents.read() {
        for mut trim in &mut boats {
            *trim = trim.toggled();
        }
    }
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

        let remaining = station.goal - station.along;
        if remaining != 0.0 {
            let travel = (speed * step).min(remaining.abs());
            station.along += travel.copysign(remaining);
            let wanted = path.tangent_at(station.along) * remaining.signum();
            heading.0 = heading.0.rotate_towards(wanted, UBOAT_TURN_RATE * step);
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
        flotilla.kills = 0;
    }
}
