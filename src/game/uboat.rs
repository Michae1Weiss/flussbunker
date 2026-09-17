//! The player's U-boat: river movement, dive state, torpedoes, rearming.
//! Sole writer of `Flotilla`, `Trim`, `Torpedoes`, `Station`.

use bevy::prelude::*;

use crate::balance::{
    rank_for, TORPEDO_RUN, TORPEDO_SPEED, UBOAT_HEALTH, UBOAT_REARM_SECONDS,
    UBOAT_REPLACEMENT_COST, UBOAT_SPEED, UBOAT_SUBMERGED_SPEED_FACTOR, UBOAT_TRIM_SECONDS, Z_UBOAT,
};
use crate::game::combat::{DamageKind, Health, Sunk};
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::input::{CommissionUBoat, MoveUBoat, ToggleTrim};
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

/// Dive state. Transitions take `UBOAT_TRIM_SECONDS`; intents during a transition are ignored.
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
}

#[derive(Component)]
pub struct Torpedoes {
    pub loaded: u8,
    /// Per-shot reload.
    pub tube: Timer,
    /// Per-torpedo resupply while docked.
    pub rearm: Timer,
}

#[derive(Component)]
pub struct Station {
    /// `RiverPath` index currently steered toward.
    pub index: usize,
    /// `RiverPath` index of the ordered destination.
    pub goal: usize,
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
    mut intents: MessageReader<MoveUBoat>,
    mut boats: Query<&mut Station, With<UBoat>>,
) {
    for intent in intents.read() {
        for mut station in &mut boats {
            station.goal = intent.station;
        }
    }
}

fn apply_trim_intents(
    mut intents: MessageReader<ToggleTrim>,
    mut boats: Query<&mut Trim, With<UBoat>>,
) {
    for _ in intents.read() {
        for mut trim in &mut boats {
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

        let target = path.points[station.index];
        let position = transform.translation.truncate();
        let to_target = target - position;
        let travel = speed * step;

        if to_target.length() <= travel {
            // The index advances only once the waypoint is reached; advancing
            // earlier parks the boat one tile short of the ordered cell.
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

/// Fires up the straight the boat sits on, only while an enemy is in that lane.
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

        let Some(upstream) = station.index.checked_sub(1) else {
            continue;
        };
        let here = path.points[station.index];
        let heading = (path.points[upstream] - here).normalize_or_zero();
        if heading == Vec2::ZERO {
            continue;
        }

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
