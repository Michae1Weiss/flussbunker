//! Shells and torpedoes in flight, plus the little splash sprites.
//!
//! Modelling decision worth stating: a projectile is **not** a child of the
//! battery that fired it. Hierarchy means "part of" — a turret is part of a
//! tower, a shell is not. It outlives the gun's aim and moves on its own, so it
//! carries a plain `Owner`-style `target: Entity` field instead of `ChildOf`.
//! Getting this wrong is how projectiles end up teleporting when a tower is
//! sold.
//!
//! The shells home. Real ones do not, and leading the target is a nicer piece
//! of maths — but a missed shot in a tower defense reads as a bug to the
//! player, not as ballistics. Readability beats simulation here.

use bevy::prelude::*;

use crate::balance::{Z_PROJECTILE, Z_TOWER};
use crate::game::combat::{Damage, DamageKind};
use crate::game_plugin::Sim;
use crate::screens::Screen;

/// Close enough to count as a hit, in pixels.
const HIT_RADIUS: f32 = 7.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (fly_and_hit.in_set(Sim::Movement), expire_splashes.in_set(Sim::Death)),
    );
}

#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f32,
    pub kind: DamageKind,
    pub speed: f32,
}

/// A short-lived decorative sprite. `Lifetime` is generic on purpose — the next
/// three visual effects will reuse it rather than each inventing a timer.
#[derive(Component)]
pub struct Lifetime(pub Timer);

pub fn projectile_bundle(
    origin: Vec2,
    target: Entity,
    damage: f32,
    kind: DamageKind,
    speed: f32,
) -> impl Bundle {
    let (color, size) = match kind {
        DamageKind::Shell => (Color::srgb(1.0, 0.92, 0.60), Vec2::new(4.0, 4.0)),
        DamageKind::Torpedo => (Color::srgb(0.75, 0.95, 1.0), Vec2::new(4.0, 10.0)),
    };
    (
        Name::new("Projectile"),
        Projectile { target, damage, kind, speed },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(origin.extend(Z_PROJECTILE)),
        DespawnOnExit(Screen::Gameplay),
    )
}

pub fn splash_bundle(position: Vec2, color: Color) -> impl Bundle {
    (
        Name::new("Splash"),
        Lifetime(Timer::from_seconds(0.18, TimerMode::Once)),
        Sprite {
            color,
            custom_size: Some(Vec2::splat(10.0)),
            ..default()
        },
        Transform::from_translation(position.extend(Z_TOWER)),
        DespawnOnExit(Screen::Gameplay),
    )
}

fn fly_and_hit(
    time: Res<Time>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut shells: Query<(Entity, &mut Transform, &Projectile)>,
    targets: Query<&Transform, Without<Projectile>>,
) {
    let step = time.delta_secs();

    for (entity, mut transform, shell) in &mut shells {
        // The target sank while this shell was in the air. Despawning is the
        // honest outcome: the alternative (re-targeting mid-flight) makes
        // damage output impossible to reason about.
        let Ok(target) = targets.get(shell.target) else {
            commands.entity(entity).despawn();
            continue;
        };

        let position = transform.translation.truncate();
        let to_target = target.translation.truncate() - position;

        if to_target.length() <= HIT_RADIUS.max(shell.speed * step) {
            damage.write(Damage {
                target: shell.target,
                amount: shell.damage,
                kind: shell.kind,
            });
            commands.spawn(splash_bundle(
                target.translation.truncate(),
                Color::srgb(1.0, 0.85, 0.55),
            ));
            commands.entity(entity).despawn();
            continue;
        }

        let heading = to_target.normalize_or_zero();
        transform.translation += (heading * shell.speed * step).extend(0.0);
        transform.rotation = Quat::from_rotation_z(heading.to_angle() - std::f32::consts::FRAC_PI_2);
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
            // Fade out. Two lines of juice, no particle system required.
            sprite.color.set_alpha(1.0 - lifetime.0.fraction());
        }
    }
}
