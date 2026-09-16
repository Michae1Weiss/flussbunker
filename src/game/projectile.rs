//! Two kinds of thing in flight, and the difference is a design decision.
//!
//! * **Shells** home. Real ones do not, but a missed shot in a tower defense
//!   reads as a bug rather than as ballistics, and shore batteries are meant to
//!   be reliable chip damage.
//! * **Torpedoes run straight.** They take a heading, not a target, and they
//!   travel until they hit a hull or lose the water. This is the whole v2
//!   redesign in one struct field:
//!     - firing over land is *structurally* impossible, no line-of-sight check;
//!     - a long straight is a firing lane and a bend is dead water, so the
//!       river's shape becomes the tactical map;
//!     - a torpedo down a packed channel can hit several boats.
//!
//! A projectile is never a child of the gun that fired it. Hierarchy means
//! "part of"; a shell outlives the aim that produced it.

use bevy::prelude::*;

use crate::balance::{Z_PROJECTILE, Z_TOWER};
use crate::game::combat::{Damage, DamageKind};
use crate::game::map::MapGrid;
use crate::game_plugin::Sim;
use crate::screens::Screen;

const HIT_RADIUS: f32 = 7.0;
/// Half a tile: a torpedo runs down the middle of the channel, so anything
/// within half a tile of the axis is in its path.
const TORPEDO_BEAM: f32 = 24.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            (fly_shells, run_torpedoes).in_set(Sim::Movement),
            expire_splashes.in_set(Sim::Death),
        ),
    );
}

/// Homing: knows *who* it is for.
#[derive(Component)]
pub struct Shell {
    pub target: Entity,
    pub damage: f32,
    pub kind: DamageKind,
    pub speed: f32,
    /// Radius of area damage on impact. Zero for single-target guns.
    pub splash: f32,
}

/// Straight-running: knows *where* it is going, and nothing about targets.
#[derive(Component)]
pub struct Torpedo {
    pub heading: Vec2,
    pub damage: f32,
    pub speed: f32,
    /// Metres of fuel left, effectively.
    pub run_remaining: f32,
}

#[derive(Component)]
pub struct Lifetime(pub Timer);

pub fn shell_bundle(
    origin: Vec2,
    target: Entity,
    damage: f32,
    kind: DamageKind,
    speed: f32,
    splash: f32,
) -> impl Bundle {
    (
        Name::new("Shell"),
        Shell {
            target,
            damage,
            kind,
            speed,
            splash,
        },
        Sprite {
            color: Color::srgb(1.0, 0.92, 0.60),
            custom_size: Some(Vec2::splat(if splash > 0.0 { 6.0 } else { 4.0 })),
            ..default()
        },
        Transform::from_translation(origin.extend(Z_PROJECTILE)),
        DespawnOnExit(Screen::Gameplay),
    )
}

pub fn torpedo_bundle(
    origin: Vec2,
    heading: Vec2,
    damage: f32,
    speed: f32,
    run: f32,
) -> impl Bundle {
    (
        Name::new("Torpedo"),
        Torpedo {
            heading,
            damage,
            speed,
            run_remaining: run,
        },
        Sprite {
            color: Color::srgb(0.80, 0.96, 1.0),
            custom_size: Some(Vec2::new(5.0, 14.0)),
            ..default()
        },
        Transform::from_translation(origin.extend(Z_PROJECTILE)).with_rotation(
            Quat::from_rotation_z(heading.to_angle() - std::f32::consts::FRAC_PI_2),
        ),
        DespawnOnExit(Screen::Gameplay),
    )
}

pub fn splash_bundle(position: Vec2, color: Color, size: f32) -> impl Bundle {
    (
        Name::new("Splash"),
        Lifetime(Timer::from_seconds(0.22, TimerMode::Once)),
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_translation(position.extend(Z_TOWER)),
        DespawnOnExit(Screen::Gameplay),
    )
}

fn fly_shells(
    time: Res<Time>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut shells: Query<(Entity, &mut Transform, &Shell)>,
    targets: Query<&Transform, Without<Shell>>,
    hittable: Query<(Entity, &Transform), (With<crate::game::enemy::Enemy>, Without<Shell>)>,
) {
    let step = time.delta_secs();

    for (entity, mut transform, shell) in &mut shells {
        // Target sank mid-flight. Despawning is the honest outcome;
        // re-targeting would make damage output impossible to reason about.
        let Ok(target) = targets.get(shell.target) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = transform.translation.truncate();
        let to_target = target.translation.truncate() - position;

        if to_target.length() <= HIT_RADIUS.max(shell.speed * step) {
            let impact = target.translation.truncate();

            if shell.splash > 0.0 {
                // Area damage: everything in the radius, including the boat we
                // aimed at. One loop, no special-casing the primary target.
                // Ref: https://subscription.packtpub.com/book/game-development/9781788392365/2/ch02lvl1sec19/consider-using-distance-squared-over-distance
                for (other, other_transform) in &hittable {
                    if impact.distance_squared(other_transform.translation.truncate())
                        <= shell.splash * shell.splash
                    {
                        damage.write(Damage {
                            target: other,
                            amount: shell.damage,
                            kind: shell.kind,
                        });
                    }
                }
                commands.spawn(splash_bundle(
                    impact,
                    Color::srgb(1.0, 0.75, 0.40),
                    shell.splash,
                ));
            } else {
                damage.write(Damage {
                    target: shell.target,
                    amount: shell.damage,
                    kind: shell.kind,
                });
                commands.spawn(splash_bundle(impact, Color::srgb(1.0, 0.85, 0.55), 10.0));
            }

            commands.entity(entity).despawn();
            continue;
        }

        let heading = to_target.normalize_or_zero();
        transform.translation += (heading * shell.speed * step).extend(0.0);
        transform.rotation =
            Quat::from_rotation_z(heading.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

/// Straight run: move, check for a hull, check we are still in water.
///
/// The water check is the entire fix for "the u-boat shoots over land", and it
/// costs one grid lookup per tick. Note that it is done on the *torpedo*, not
/// as a line-of-sight test at firing time — which means a torpedo fired down a
/// legal lane still dies at the bend, exactly as a player watching it would
/// expect.
fn run_torpedoes(
    time: Res<Time>,
    grid: Res<MapGrid>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut torpedoes: Query<(Entity, &mut Transform, &mut Torpedo)>,
    enemies: Query<(Entity, &Transform), (With<crate::game::enemy::Enemy>, Without<Torpedo>)>,
) {
    let step = time.delta_secs();

    for (entity, mut transform, mut torpedo) in &mut torpedoes {
        let travel = torpedo.speed * step;
        transform.translation += (torpedo.heading * travel).extend(0.0);
        torpedo.run_remaining -= travel;

        let position = transform.translation.truncate();

        let out_of_water = !grid
            .tile(grid.world_to_grid(position))
            .is_some_and(|t| t.is_navigable());
        if torpedo.run_remaining <= 0.0 || out_of_water {
            commands.spawn(splash_bundle(position, Color::srgb(0.7, 0.85, 0.95), 8.0));
            commands.entity(entity).despawn();
            continue;
        }

        // First hull within the beam. `min_by` on distance so a torpedo passing
        // two overlapping boats hits the near one.
        let hit = enemies
            .iter()
            .filter(|(_, enemy)| {
                position.distance_squared(enemy.translation.truncate())
                    <= TORPEDO_BEAM * TORPEDO_BEAM
            })
            .min_by(|a, b| {
                position
                    .distance_squared(a.1.translation.truncate())
                    .total_cmp(&position.distance_squared(b.1.translation.truncate()))
            });

        if let Some((target, enemy)) = hit {
            damage.write(Damage {
                target,
                amount: torpedo.damage,
                kind: DamageKind::Torpedo,
            });
            // Torpedo hits get a bigger, whiter splash than shells. The best
            // moment in the game should look like the best moment in the game.
            commands.spawn(splash_bundle(
                enemy.translation.truncate(),
                Color::srgb(0.95, 1.0, 1.0),
                34.0,
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn expire_splashes(
    time: Res<Time>,
    mut commands: Commands,
    mut splashes: Query<(Entity, &mut Lifetime, &mut Sprite)>,
) {
    for (entity, mut lifetime, mut sprite) in &mut splashes {
        if lifetime.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        } else {
            sprite.color.set_alpha(1.0 - lifetime.0.fraction());
        }
    }
}
