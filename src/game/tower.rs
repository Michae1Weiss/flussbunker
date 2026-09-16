//! Shore batteries: acquire a target, pull the trigger.
//!
//! No physics engine, no spatial index, no colliders. "Is that boat in range"
//! is `distance_squared <= range * range` — one comparison, no square root, and
//! it stays correct at any frame rate. A tower defense with a few hundred
//! entities does not need a broadphase; adding one before a profiler says so is
//! complexity bought with nothing.

use bevy::prelude::*;

use crate::balance::{TowerKind, Z_TOWER};
use crate::game::economy::Treasury;
use crate::game::map::{MapGrid, Tile};
use crate::game::orders::BuildTower;
use crate::game::combat::DamageKind;
use crate::game::enemy::{Enemy, PathProgress};
use crate::game::projectile::shell_bundle;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (construct.in_set(Sim::Spawn), fire_at_leader.in_set(Sim::Targeting)),
    );
}

#[derive(Component)]
pub struct Tower;

#[derive(Component, Clone, Copy, Debug)]
pub struct TowerType(pub TowerKind);

#[derive(Component)]
pub struct Range(pub f32);

#[derive(Component)]
pub struct Gun {
    pub damage: f32,
    pub kind: DamageKind,
    pub projectile_speed: f32,
    /// Impact radius. Zero means single target — the mortar is the reason this
    /// exists, and the reason a swarm wave has an answer that is not "more Flak".
    pub splash: f32,
    pub reload: Timer,
}

pub fn tower_bundle(kind: TowerKind, position: Vec2) -> impl Bundle {
    let stats = kind.stats();
    (
        Name::new(stats.name),
        Tower,
        TowerType(kind),
        Range(stats.range),
        Gun {
            damage: stats.damage,
            kind: stats.damage_kind,
            projectile_speed: stats.projectile_speed,
            splash: stats.splash,
            // `Repeating` and not `Once`: a battery that reloads forever needs
            // no state machine, just a timer that keeps wrapping.
            reload: Timer::from_seconds(stats.reload, TimerMode::Repeating),
        },
        Sprite {
            color: Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2]),
            custom_size: Some(Vec2::splat(crate::balance::TILE_SIZE * 0.62)),
            ..default()
        },
        Transform::from_translation(position.extend(Z_TOWER)),
        DespawnOnExit(Screen::Gameplay),
    )
}

/// Target the boat **furthest along the river**, not the nearest one.
///
/// This is a gameplay decision hiding in a `min_by`. "Nearest" makes towers
/// re-target constantly and lets leakers through while guns chew on fresh
/// arrivals. "First" (closest to the bunker) is the standard tower-defense
/// default because it concentrates damage on the boat that is about to cost you
/// integrity. Bloons exposes this as a per-tower setting; it is a genuinely
/// interesting knob and a good second-iteration feature.
fn fire_at_leader(
    time: Res<Time>,
    mut commands: Commands,
    mut towers: Query<(&Transform, &Range, &mut Gun), With<Tower>>,
    enemies: Query<(Entity, &Transform, &PathProgress), With<Enemy>>,
) {
    for (tower_transform, range, mut gun) in &mut towers {
        if !gun.reload.tick(time.delta()).just_finished() {
            continue;
        }

        let origin = tower_transform.translation.truncate();
        let target = enemies
            .iter()
            .filter(|(_, transform, _)| {
                origin.distance_squared(transform.translation.truncate()) <= range.0 * range.0
            })
            .max_by_key(|(_, _, progress)| progress.next);

        let Some((entity, _, _)) = target else {
            // Nothing in range. Reset so the next boat entering range is shot
            // immediately instead of waiting out a phantom reload — small
            // thing, big difference in how responsive the battery feels.
            gun.reload.reset();
            continue;
        };

        commands.spawn(shell_bundle(
            origin,
            entity,
            gun.damage,
            gun.kind,
            gun.projectile_speed,
            gun.splash,
        ));
    }
}

/// Consumes build intents. Cost, legality and placement rules live here, with
/// the thing being built — `orders.rs` only knows that a click happened.
///
/// Every rejection is a silent `continue`. In a shipped game each one would
/// play a different "denied" sound, which is where the player actually learns
/// the rules; the branches are already separated for exactly that reason.
fn construct(
    mut requests: MessageReader<BuildTower>,
    mut commands: Commands,
    mut treasury: ResMut<Treasury>,
    grid: Res<MapGrid>,
    towers: Query<&Transform, With<Tower>>,
) {
    for request in requests.read() {
        if grid.tile(request.cell) != Some(Tile::Land) {
            continue;
        }
        let occupied = towers
            .iter()
            .any(|t| grid.world_to_grid(t.translation.truncate()) == request.cell);
        if occupied {
            continue;
        }
        if !treasury.try_spend(request.kind.stats().cost) {
            continue;
        }
        commands.spawn(tower_bundle(request.kind, grid.grid_to_world(request.cell)));
    }
}
