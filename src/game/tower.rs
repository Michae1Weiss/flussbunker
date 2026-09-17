//! Shore batteries: construction from `BuildTower` on build spots, target selection, firing.
//! Sole writer of `Tower`, `TowerType`, `Range`, `Gun`, `Occupied`.

use bevy::prelude::*;

use crate::balance::{TowerKind, TILE_SIZE, Z_TOWER};
use crate::game::combat::DamageKind;
use crate::game::economy::Treasury;
use crate::game::enemy::{Along, Enemy};
use crate::game::input::BuildTower;
use crate::game::map::BuildSpot;
use crate::game::projectile::shell_bundle;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            construct.in_set(Sim::Spawn),
            fire_at_leader.in_set(Sim::Targeting),
        ),
    );
}

#[derive(Component)]
pub struct Tower;

/// On a `BuildSpot` that holds this turret.
#[derive(Component)]
pub struct Occupied(pub Entity);

#[derive(Component, Clone, Copy, Debug)]
pub struct TowerType(pub TowerKind);

/// World units.
#[derive(Component)]
pub struct Range(pub f32);

#[derive(Component)]
pub struct Gun {
    pub damage: f32,
    pub kind: DamageKind,
    pub projectile_speed: f32,
    /// Impact radius in world units; 0 = single target.
    pub splash: f32,
    /// Ready when finished; reset after each shot.
    pub reload: Timer,
}

pub fn tower_bundle(kind: TowerKind, position: Vec2) -> impl Bundle {
    let stats = kind.stats();
    let mut reload = Timer::from_seconds(stats.reload, TimerMode::Once);
    reload.set_elapsed(reload.duration());
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
            reload,
        },
        Sprite {
            color: Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2]),
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.62)),
            ..default()
        },
        Transform::from_translation(position.extend(Z_TOWER)),
        DespawnOnExit(Screen::Gameplay),
    )
}

/// Targets the enemy in range that is furthest along the river.
fn fire_at_leader(
    time: Res<Time>,
    mut commands: Commands,
    mut towers: Query<(&Transform, &Range, &mut Gun), With<Tower>>,
    enemies: Query<(Entity, &Transform, &Along), With<Enemy>>,
) {
    for (tower_transform, range, mut gun) in &mut towers {
        gun.reload.tick(time.delta());
        if !gun.reload.is_finished() {
            continue;
        }

        let origin = tower_transform.translation.truncate();
        let target = enemies
            .iter()
            .filter(|(_, transform, _)| {
                origin.distance_squared(transform.translation.truncate()) <= range.0 * range.0
            })
            .max_by(|a, b| a.2 .0.total_cmp(&b.2 .0));

        let Some((entity, _, _)) = target else {
            continue;
        };

        gun.reload.reset();
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

fn construct(
    mut requests: MessageReader<BuildTower>,
    mut commands: Commands,
    mut treasury: ResMut<Treasury>,
    spots: Query<(&Transform, Has<Occupied>), With<BuildSpot>>,
) {
    for request in requests.read() {
        let Ok((transform, occupied)) = spots.get(request.spot) else {
            continue;
        };
        if occupied || !treasury.try_spend(request.kind.stats().cost) {
            continue;
        }
        let tower = commands
            .spawn(tower_bundle(request.kind, transform.translation.truncate()))
            .id();
        commands.entity(request.spot).insert(Occupied(tower));
    }
}
