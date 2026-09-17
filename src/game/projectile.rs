//! Shells (tracked, always hit) and torpedoes (straight run, first hull on the line).
//! Sole writer of `Shell`, `Torpedo`, `Lifetime`; writes `Damage` and `TorpedoLost`.

use bevy::prelude::*;

use crate::balance::{Z_PROJECTILE, Z_TOWER};
use crate::game::combat::{Damage, DamageKind, Weapon};
use crate::game::map::RiverPath;
use crate::game_plugin::Sim;
use crate::screens::Screen;

const HIT_RADIUS: f32 = 7.0;
/// Hulls within this distance of the run line are hit.
const TORPEDO_BEAM: f32 = 24.0;

pub fn plugin(app: &mut App) {
    app.add_message::<TorpedoLost>().add_systems(
        FixedUpdate,
        (
            (fly_shells, run_torpedoes).in_set(Sim::Movement),
            expire_splashes.in_set(Sim::Death),
        ),
    );
}

#[derive(Component)]
pub struct Shell {
    pub target: Entity,
    pub damage: f32,
    pub ap: bool,
    pub speed: f32,
    pub weapon: Weapon,
}

#[derive(Component)]
pub struct Torpedo {
    pub heading: Vec2,
    pub damage: f32,
    pub speed: f32,
    /// World units left before the run ends.
    pub run_remaining: f32,
}

#[derive(Component)]
pub struct Lifetime(pub Timer);

/// A torpedo ran out or left the channel without hitting anything.
#[derive(Message)]
pub struct TorpedoLost {
    pub at: Vec2,
}

pub fn shell_bundle(
    origin: Vec2,
    target: Entity,
    damage: f32,
    ap: bool,
    speed: f32,
    weapon: Weapon,
) -> impl Bundle {
    (
        Name::new("Shell"),
        Shell {
            target,
            damage,
            ap,
            speed,
            weapon,
        },
        Sprite {
            color: Color::srgb(1.0, 0.92, 0.60),
            custom_size: Some(Vec2::splat(4.0)),
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
) {
    let step = time.delta_secs();

    for (entity, mut transform, shell) in &mut shells {
        // Target gone mid-flight: despawn, never re-target.
        let Ok(target) = targets.get(shell.target) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = transform.translation.truncate();
        let to_target = target.translation.truncate() - position;

        if to_target.length() <= HIT_RADIUS.max(shell.speed * step) {
            let impact = target.translation.truncate();
            damage.write(Damage {
                target: shell.target,
                amount: shell.damage,
                kind: DamageKind::Shell,
                ap: shell.ap,
                weapon: shell.weapon,
            });
            commands.spawn(splash_bundle(impact, Color::srgb(1.0, 0.85, 0.55), 10.0));
            commands.entity(entity).despawn();
            continue;
        }

        let heading = to_target.normalize_or_zero();
        transform.translation += (heading * shell.speed * step).extend(0.0);
        transform.rotation =
            Quat::from_rotation_z(heading.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

fn run_torpedoes(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut lost: MessageWriter<TorpedoLost>,
    mut torpedoes: Query<(Entity, &mut Transform, &mut Torpedo)>,
    enemies: Query<(Entity, &Transform), (With<crate::game::enemy::Enemy>, Without<Torpedo>)>,
) {
    let step = time.delta_secs();

    for (entity, mut transform, mut torpedo) in &mut torpedoes {
        let travel = torpedo.speed * step;
        transform.translation += (torpedo.heading * travel).extend(0.0);
        torpedo.run_remaining -= travel;

        let position = transform.translation.truncate();

        if torpedo.run_remaining <= 0.0 || !path.contains(position) {
            lost.write(TorpedoLost { at: position });
            commands.spawn(splash_bundle(position, Color::srgb(0.7, 0.85, 0.95), 8.0));
            commands.entity(entity).despawn();
            continue;
        }

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
                ap: false,
                weapon: Weapon::Torpedo,
            });
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
