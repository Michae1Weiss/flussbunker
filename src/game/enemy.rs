//! British boats: how they are built, how they move, how they hurt you.
//!
//! `enemy_bundle` is a **bundle function**, not a `#[require]` web. When five
//! components are spawned together as one conceptual thing and none of them
//! owns the others, one function that returns `impl Bundle` reads better than
//! five type-level requirements. Required components earn their keep for
//! invariants ("an Enemy always has a Transform"); a constructor earns its keep
//! for composition. Both are idiomatic; pick by which question you are
//! answering.

use bevy::prelude::*;

use crate::balance::{EnemyKind, Z_ENEMY};
use crate::game::combat::{Armor, Damage, DamageKind, Health};
use crate::game::map::RiverPath;
use crate::game::uboat::UBoat;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_message::<EnemyReachedBunker>().add_systems(
        FixedUpdate,
        (
            advance_along_river.in_set(Sim::Movement),
            drop_depth_charges.in_set(Sim::Targeting),
        ),
    );
}

#[derive(Component)]
pub struct Enemy;

/// Wraps the balance-table enum so it can be queried as a component.
///
/// Why not `#[derive(Component)]` on `EnemyKind` itself: that would make
/// `balance.rs` depend on Bevy. Keeping the tuning table free of engine types
/// means a designer can read it, and a future RON loader can produce it,
/// without either touching the ECS. One newtype is a cheap price.
#[derive(Component, Clone, Copy, Debug)]
pub struct EnemyType(pub EnemyKind);

#[derive(Component)]
pub struct Speed(pub f32);

/// How far along `RiverPath` this boat is. `next` is the waypoint it is
/// steering towards, which also doubles as "how close to the bunker" for tower
/// target priority.
#[derive(Component, Default)]
pub struct PathProgress {
    pub next: usize,
}

#[derive(Component)]
pub struct Bounty(pub u32);

/// Integrity lost when this boat reaches the bunker.
#[derive(Component)]
pub struct BreachDamage(pub u32);

/// Destroyer anti-submarine armament.
#[derive(Component)]
pub struct DepthCharges {
    pub range: f32,
    pub damage: f32,
    pub reload: Timer,
}

/// A boat got through. Written here (the producer), read by `bunker.rs`.
///
/// Producer-owned messages keep the dependency arrows pointing one way: the
/// bunker knows about enemies, enemies know nothing about the bunker. The
/// alternative — a shared `events.rs` everyone imports — is fine too, and
/// becomes the better choice once several features both produce and consume
/// the same signal.
#[derive(Message)]
pub struct EnemyReachedBunker {
    pub damage: u32,
}

/// Everything an enemy is, in one place.
pub fn enemy_bundle(kind: EnemyKind, spawn: Vec2) -> impl Bundle {
    let stats = kind.stats();
    (
        Name::new(stats.name),
        Enemy,
        EnemyType(kind),
        Health::new(stats.health),
        Armor(stats.armor),
        Speed(stats.speed),
        Bounty(stats.bounty),
        BreachDamage(stats.breach_damage),
        PathProgress::default(),
        Sprite {
            color: Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2]),
            // Long and thin: a boat should read as a boat and, once rotated,
            // show its heading at a glance. Readability before decoration.
            custom_size: Some(Vec2::new(stats.size * 0.55, stats.size)),
            ..default()
        },
        Transform::from_translation(spawn.extend(Z_ENEMY)),
        DespawnOnExit(Screen::Gameplay),
    )
}

/// Adds the destroyer's depth charges. Kept out of `enemy_bundle` because it is
/// conditional: bundles are for what is always true.
pub fn depth_charge_bundle(kind: EnemyKind) -> Option<DepthCharges> {
    kind.stats().depth_charge.map(|dc| DepthCharges {
        range: dc.range,
        damage: dc.damage,
        reload: Timer::from_seconds(dc.reload, TimerMode::Repeating),
    })
}

/// Steer towards the next waypoint; on arrival, take the next one.
///
/// Runs in `FixedUpdate`, so the simulation is identical at 30 fps and 240 fps.
/// The visible cost is that `Transform` updates 64 times a second while the
/// screen may refresh 144 — on slow boats it is imperceptible. When it starts
/// to matter, the fix is `bevy_transform_interpolation`, not moving this system
/// to `Update`.
fn advance_along_river(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    mut breach: MessageWriter<EnemyReachedBunker>,
    mut boats: Query<(Entity, &mut Transform, &Speed, &mut PathProgress, &BreachDamage), With<Enemy>>,
) {
    let step = time.delta_secs();

    for (entity, mut transform, speed, mut progress, breach_damage) in &mut boats {
        let Some(target) = path.points.get(progress.next).copied() else {
            // Ran off the end of the path: the bunker has been reached.
            breach.write(EnemyReachedBunker { damage: breach_damage.0 });
            commands.entity(entity).despawn();
            continue;
        };

        let position = transform.translation.truncate();
        let to_target = target - position;
        let travel = speed.0 * step;

        if to_target.length() <= travel {
            // Snap to the waypoint and aim at the next one. Snapping loses a
            // few pixels of travel per corner; at this speed nobody can see it,
            // and the alternative (carrying the remainder around corners) is
            // three times the code for zero visible difference.
            transform.translation = target.extend(transform.translation.z);
            progress.next += 1;
        } else {
            transform.translation += (to_target.normalize_or_zero() * travel).extend(0.0);
        }

        // Point the sprite along its heading. The sprite is drawn tall, so
        // "up" is the bow: rotate by the heading angle minus 90 degrees.
        if to_target.length_squared() > f32::EPSILON {
            transform.rotation = Quat::from_rotation_z(to_target.to_angle() - std::f32::consts::FRAC_PI_2);
        }
    }
}

/// Destroyers fight back. This is what makes a u-boat sortie a *decision*
/// rather than a free button: send it too early into a screening destroyer and
/// you lose 110 marks and a pen slot.
fn drop_depth_charges(
    time: Res<Time>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut destroyers: Query<(&Transform, &mut DepthCharges)>,
    uboats: Query<(Entity, &Transform), With<UBoat>>,
) {
    for (transform, mut charges) in &mut destroyers {
        if !charges.reload.tick(time.delta()).just_finished() {
            continue;
        }
        let origin = transform.translation.truncate();
        let in_range = uboats.iter().find(|(_, boat)| {
            origin.distance_squared(boat.translation.truncate()) <= charges.range * charges.range
        });

        if let Some((target, boat)) = in_range {
            damage.write(Damage {
                target,
                amount: charges.damage,
                kind: DamageKind::Shell,
            });
            // Splash marker, despawned by its own timer in `projectile.rs`.
            commands.spawn(crate::game::projectile::splash_bundle(
                boat.translation.truncate(),
                Color::srgb(0.85, 0.85, 0.95),
            ));
        }
    }
}
