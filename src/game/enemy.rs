//! Enemy boats: movement along the river in convoy, breaching, destroyer gunfire.
//! Sole writer of `Along`, `Cruising`, `Shelling`; writes `EnemyReachedBunker` and `Damage`.
//! Rules: docs/design/enemies.md

use bevy::prelude::*;

use crate::balance::{EnemyKind, CONVOY_GAP, ENEMY_TURN_RATE, PROLOGUE_CRUISE, TILE_SIZE, Z_ENEMY};
use crate::game::combat::{Armor, Health, Weapon};
use crate::game::map::RiverPath;
use crate::game::projectile::{shell_bundle, splash_bundle};
use crate::game::uboat::{Trim, UBoat};
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_message::<EnemyReachedBunker>().add_systems(
        FixedUpdate,
        (
            (advance_in_convoy, cruise).in_set(Sim::Movement),
            shell_the_boat.in_set(Sim::Targeting),
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

/// Hull length in world units; the convoy gap and ramming reach use it.
#[derive(Component)]
pub struct Hull(pub f32);

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
pub struct Shelling {
    pub range: f32,
    pub damage: f32,
    /// Near misses until finished; reset whenever contact is lost.
    pub ranging: Timer,
    pub reload: Timer,
}

/// Sails up and down between two arc lengths instead of toward the bunker.
#[derive(Component)]
pub struct Cruising {
    pub low: f32,
    pub high: f32,
    pub downstream: bool,
}

impl Cruising {
    pub fn prologue(path: &RiverPath) -> Self {
        Self {
            low: path.length() * PROLOGUE_CRUISE.0,
            high: path.length() * PROLOGUE_CRUISE.1,
            downstream: true,
        }
    }
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
        Hull(stats.size),
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

pub fn shelling_bundle(kind: EnemyKind) -> Option<Shelling> {
    kind.stats().shelling.map(|s| Shelling {
        range: s.range,
        damage: s.damage,
        ranging: Timer::from_seconds(s.ranging, TimerMode::Once),
        reload: Timer::from_seconds(s.reload, TimerMode::Repeating),
    })
}

/// Ships are timetables, except that none overtakes a slower one ahead.
fn advance_in_convoy(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    mut breach: MessageWriter<EnemyReachedBunker>,
    mut boats: Query<
        (
            Entity,
            &mut Transform,
            &Speed,
            &Hull,
            &mut Along,
            &BreachDamage,
        ),
        (With<Enemy>, Without<Cruising>),
    >,
) {
    let step = time.delta_secs();
    let mut order: Vec<(Entity, f32, f32)> = boats
        .iter()
        .map(|(entity, _, _, hull, along, _)| (entity, along.0, hull.0))
        .collect();
    order.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut ahead: Option<(f32, f32)> = None;
    for (entity, _, hull) in order {
        let (_, mut transform, speed, _, mut along, breach_damage) = boats.get_mut(entity).unwrap();
        let next = convoy_limit(along.0 + speed.0 * step, hull, ahead);
        along.0 = next.max(along.0);
        ahead = Some((along.0, hull));

        if along.0 >= path.length() {
            breach.write(EnemyReachedBunker {
                damage: breach_damage.0,
            });
            commands.spawn(splash_bundle(
                path.bunker_point(),
                Color::srgb(1.0, 0.5, 0.3),
                TILE_SIZE * (0.8 + 0.2 * breach_damage.0 as f32),
            ));
            commands.entity(entity).despawn();
            continue;
        }

        let z = transform.translation.z;
        transform.translation = path.point_at(along.0).extend(z);
        // Sprites are drawn bow-up, hence the -90°.
        let bow = (transform.rotation * Vec3::Y).truncate();
        let heading = bow.rotate_towards(path.tangent_at(along.0), ENEMY_TURN_RATE * step);
        transform.rotation =
            Quat::from_rotation_z(heading.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

fn cruise(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut ships: Query<(&mut Transform, &Speed, &mut Along, &mut Cruising), With<Enemy>>,
) {
    let step = time.delta_secs();
    for (mut transform, speed, mut along, mut cruising) in &mut ships {
        let direction = if cruising.downstream { 1.0 } else { -1.0 };
        along.0 += speed.0 * step * direction;
        if along.0 >= cruising.high {
            cruising.downstream = false;
        } else if along.0 <= cruising.low {
            cruising.downstream = true;
        }
        let z = transform.translation.z;
        transform.translation = path.point_at(along.0).extend(z);
        let bow = (transform.rotation * Vec3::Y).truncate();
        let heading =
            bow.rotate_towards(path.tangent_at(along.0) * direction, ENEMY_TURN_RATE * step);
        transform.rotation =
            Quat::from_rotation_z(heading.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

/// Where a ship with `hull` may advance to, given the ship ahead as (along, hull).
fn convoy_limit(wanted: f32, hull: f32, ahead: Option<(f32, f32)>) -> f32 {
    match ahead {
        Some((lead_along, lead_hull)) => {
            wanted.min(lead_along - (lead_hull + hull) * 0.5 - CONVOY_GAP)
        }
        None => wanted,
    }
}

/// Near misses beside the boat while ranging in, then tracked shells every reload.
fn shell_the_boat(
    time: Res<Time>,
    mut commands: Commands,
    mut gunners: Query<(&Transform, &mut Shelling)>,
    uboats: Query<(Entity, &Transform, &Trim), With<UBoat>>,
) {
    for (transform, mut shelling) in &mut gunners {
        let origin = transform.translation.truncate();
        let contact = uboats.iter().find(|(_, boat, trim)| {
            trim.is_hittable()
                && origin.distance_squared(boat.translation.truncate())
                    <= shelling.range * shelling.range
        });
        let Some((target, boat, _)) = contact else {
            shelling.ranging.reset();
            continue;
        };

        let boat_position = boat.translation.truncate();
        if !shelling.ranging.tick(time.delta()).is_finished() {
            if shelling.reload.tick(time.delta()).just_finished() {
                let volley = (shelling.ranging.elapsed_secs()
                    / shelling.reload.duration().as_secs_f32()) as i32;
                let side = if volley % 2 == 0 { 1.0 } else { -1.0 };
                let beside =
                    (boat_position - origin).perp().normalize_or_zero() * side * TILE_SIZE * 0.7;
                commands.spawn(splash_bundle(
                    boat_position + beside,
                    Color::srgb(0.85, 0.9, 1.0),
                    18.0,
                ));
            }
            continue;
        }

        if shelling.reload.tick(time.delta()).just_finished() {
            commands.spawn(shell_bundle(
                origin,
                target,
                shelling.damage,
                false,
                700.0,
                Weapon::Enemy,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ship_never_passes_the_one_ahead() {
        let lead = Some((100.0, 20.0));
        assert_eq!(convoy_limit(50.0, 20.0, lead), 50.0);
        assert_eq!(convoy_limit(500.0, 20.0, lead), 100.0 - 20.0 - CONVOY_GAP);
    }

    #[test]
    fn the_lead_ship_is_unconstrained() {
        assert_eq!(convoy_limit(500.0, 20.0, None), 500.0);
    }
}
