//! Enemy boats: movement along the river, breaching, destroyer depth charges.
//! Sole writer of `Along`, `DepthCharges`; writes `EnemyReachedBunker` and `Damage`.

use bevy::prelude::*;

use crate::balance::{EnemyKind, Z_ENEMY};
use crate::game::combat::{Armor, Damage, DamageKind, Health};
use crate::game::map::RiverPath;
use crate::game::projectile::splash_bundle;
use crate::game::uboat::{Trim, UBoat};
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
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

/// Newtype so `balance.rs` stays free of Bevy types.
#[derive(Component, Clone, Copy, Debug)]
pub struct EnemyType(pub EnemyKind);

/// World units per second.
#[derive(Component)]
pub struct Speed(pub f32);

/// Arc length along `RiverPath`; doubles as tower target priority.
#[derive(Component, Default)]
pub struct Along(pub f32);

/// RM paid on sinking.
#[derive(Component)]
pub struct Bounty(pub u32);

/// Integrity lost when this boat reaches the bunker.
#[derive(Component)]
pub struct BreachDamage(pub u32);

#[derive(Component)]
pub struct DepthCharges {
    pub range: f32,
    pub damage: f32,
    /// Warning window: contact is drawn for this long before a charge drops;
    /// reset whenever contact is lost.
    pub acquire: Timer,
}

#[derive(Message)]
pub struct EnemyReachedBunker {
    pub damage: u32,
}

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
        Along::default(),
        Sprite {
            color: Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2]),
            custom_size: Some(Vec2::new(stats.size * 0.55, stats.size)),
            ..default()
        },
        Transform::from_translation(spawn.extend(Z_ENEMY)),
        DespawnOnExit(Screen::Gameplay),
    )
}

pub fn depth_charge_bundle(kind: EnemyKind) -> Option<DepthCharges> {
    kind.stats().depth_charge.map(|dc| DepthCharges {
        range: dc.range,
        damage: dc.damage,
        acquire: Timer::from_seconds(dc.acquire, TimerMode::Repeating),
    })
}

fn advance_along_river(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    mut breach: MessageWriter<EnemyReachedBunker>,
    mut boats: Query<(Entity, &mut Transform, &Speed, &mut Along, &BreachDamage), With<Enemy>>,
) {
    for (entity, mut transform, speed, mut along, breach_damage) in &mut boats {
        along.0 += speed.0 * time.delta_secs();
        if along.0 >= path.length() {
            breach.write(EnemyReachedBunker {
                damage: breach_damage.0,
            });
            commands.entity(entity).despawn();
            continue;
        }

        let z = transform.translation.z;
        transform.translation = path.point_at(along.0).extend(z);
        // Sprites are drawn bow-up, hence the -90°.
        transform.rotation = Quat::from_rotation_z(
            path.tangent_at(along.0).to_angle() - std::f32::consts::FRAC_PI_2,
        );
    }
}

fn drop_depth_charges(
    time: Res<Time>,
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut damage: MessageWriter<Damage>,
    mut hunters: Query<(&Transform, &mut DepthCharges)>,
    uboats: Query<(Entity, &Transform, &Trim), With<UBoat>>,
) {
    for (transform, mut charges) in &mut hunters {
        let origin = transform.translation.truncate();

        let contact = uboats.iter().find(|(_, boat, trim)| {
            trim.is_hittable()
                && origin.distance_squared(boat.translation.truncate())
                    <= charges.range * charges.range
        });

        let Some((target, boat, _)) = contact else {
            charges.acquire.reset();
            continue;
        };

        charges.acquire.tick(time.delta());

        let heat = charges.acquire.fraction();
        gizmos.line_2d(
            origin,
            boat.translation.truncate(),
            Color::srgb(0.4 + 0.6 * heat, 0.15, 0.15),
        );

        if charges.acquire.just_finished() {
            damage.write(Damage {
                target,
                amount: charges.damage,
                kind: DamageKind::Shell,
            });
            commands.spawn(splash_bundle(
                boat.translation.truncate(),
                Color::srgb(0.9, 0.9, 1.0),
                26.0,
            ));
        }
    }
}
