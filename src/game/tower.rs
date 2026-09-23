//! Shore batteries on build spots: construction, upgrades, selling, targeting, firing, and
//! the barrel that turns and recoils. Sole writer of `Tower`, `TowerType`, `Tier`,
//! `Investment`, `Targeting`, `Range`, `Gun`, `Aim`, `Constructing`, `Occupied`, `Barrel` and
//! of the barrel's `Transform`. Rules: docs/design/turrets.md

use bevy::prelude::*;

use bevy::sprite::Anchor;

use crate::balance::{
    TowerKind, BARREL_TURN_RATE, RECOIL_DISTANCE, RECOIL_SECONDS, SELL_REFUND, TILE_SIZE,
    TOWER_BUILD_SECONDS, TOWER_SIZE, TOWER_TIERS, Z_TOWER,
};
use crate::game::combat::{Fired, Health, Weapon};
use crate::game::economy::Treasury;
use crate::game::enemy::{Along, Enemy};
use crate::game::input::{BuildTower, CycleTargeting, SellTower, UpgradeTower};
use crate::game::map::BuildSpot;
use crate::game::palette::INK;
use crate::game::projectile::shell_bundle;
use crate::game_plugin::{Frame, Sim};
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            (
                construct,
                finish_construction,
                upgrade,
                sell,
                cycle_targeting,
            )
                .in_set(Sim::Spawn),
            (aim, fire).chain().in_set(Sim::Targeting),
        ),
    )
    .add_systems(
        Update,
        (
            draw_construction,
            (fit_barrels, kick_barrels, turn_barrels).chain(),
        )
            .in_set(Frame::Present),
    );
}

#[derive(Component)]
pub struct Tower;

/// On a `BuildSpot` that holds this turret.
#[derive(Component)]
pub struct Occupied(pub Entity);

#[derive(Component, Clone, Copy, Debug)]
pub struct TowerType(pub TowerKind);

/// The enemy this turret shoots at, chosen every tick by `aim`; `None` when nothing is in range.
#[derive(Component, Default)]
pub struct Aim(pub Option<Entity>);

/// The part of an emplacement that turns and recoils; a child of its `Tower`, drawn from the
/// tower's centre towards local +y. `recoil` is world units pushed back, easing to 0.
#[derive(Component, Default)]
pub struct Barrel {
    pub recoil: f32,
}

/// Present while the turret is being built; it does not fire until removed.
#[derive(Component)]
pub struct Constructing(pub Timer);

/// Upgrade level, below `TOWER_TIERS`.
#[derive(Component, Clone, Copy, Debug)]
pub struct Tier(pub u8);

/// RM spent on this turret; `SELL_REFUND` of it comes back on selling.
#[derive(Component, Debug)]
pub struct Investment(pub u32);

/// World units.
#[derive(Component)]
pub struct Range(pub f32);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Targeting {
    /// Furthest along the river.
    #[default]
    First,
    Strongest,
    Weakest,
    Nearest,
}

impl Targeting {
    pub fn next(self) -> Self {
        match self {
            Targeting::First => Targeting::Strongest,
            Targeting::Strongest => Targeting::Weakest,
            Targeting::Weakest => Targeting::Nearest,
            Targeting::Nearest => Targeting::First,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Targeting::First => "Vorderstes",
            Targeting::Strongest => "Stärkstes",
            Targeting::Weakest => "Schwächstes",
            Targeting::Nearest => "Nächstes",
        }
    }
}

#[derive(Component)]
pub struct Gun {
    pub damage: f32,
    pub ap: bool,
    pub projectile_speed: f32,
    /// Shells per volley.
    pub barrels: u8,
    /// Ready when finished; reset after each volley.
    pub reload: Timer,
}

fn gun_for(kind: TowerKind, tier: u8) -> Gun {
    let stats = kind.stats(tier);
    let mut reload = Timer::from_seconds(stats.reload, TimerMode::Once);
    reload.set_elapsed(reload.duration());
    Gun {
        damage: stats.damage,
        ap: stats.ap,
        projectile_speed: stats.projectile_speed,
        barrels: stats.barrels,
        reload,
    }
}

/// Greybox barrel: a stick from the centre to the rim. `art` resizes it when a drawing exists.
fn barrel_bundle(tier: u8) -> impl Bundle {
    (
        Name::new("Barrel"),
        Barrel::default(),
        Sprite {
            color: INK,
            custom_size: Some(Vec2::new(4.0, TOWER_SIZE[tier as usize] * 0.55)),
            ..default()
        },
        Anchor::BOTTOM_CENTER,
        Transform::from_translation(Vec3::Z * 0.5),
    )
}

/// Greybox colour and size for a tier; the image, if any, belongs to `art` and is kept.
fn dress(sprite: &mut Sprite, kind: TowerKind, tier: u8, alpha: f32) {
    let stats = kind.stats(tier);
    sprite.color = if sprite.image == Handle::default() {
        Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2])
    } else {
        Color::WHITE
    }
    .with_alpha(alpha);
    sprite.custom_size = Some(Vec2::splat(TOWER_SIZE[tier as usize]));
}

pub fn tower_bundle(kind: TowerKind, position: Vec2) -> impl Bundle {
    let stats = kind.stats(0);
    (
        Name::new(stats.name),
        Tower,
        TowerType(kind),
        Tier(0),
        Investment(stats.cost),
        Targeting::default(),
        Aim::default(),
        Range(stats.range),
        gun_for(kind, 0),
        Constructing(Timer::from_seconds(TOWER_BUILD_SECONDS, TimerMode::Once)),
        {
            let mut sprite = Sprite::default();
            dress(&mut sprite, kind, 0, 0.4);
            sprite
        },
        Transform::from_translation(position.extend(Z_TOWER)),
        DespawnOnExit(Screen::Gameplay),
        children![barrel_bundle(0)],
    )
}

/// One enemy in reach of a turret, as `pick_target` sees it.
pub struct Candidate {
    pub entity: Entity,
    pub position: Vec2,
    /// Arc length along the river; larger is closer to the bunker.
    pub along: f32,
    pub health: f32,
}

pub fn pick_target(
    origin: Vec2,
    range: f32,
    targeting: Targeting,
    candidates: impl IntoIterator<Item = Candidate>,
) -> Option<Entity> {
    let in_range = candidates
        .into_iter()
        .filter(|c| origin.distance_squared(c.position) <= range * range);
    let best = match targeting {
        Targeting::First => in_range.max_by(|a, b| a.along.total_cmp(&b.along)),
        Targeting::Strongest => in_range.max_by(|a, b| a.health.total_cmp(&b.health)),
        Targeting::Weakest => in_range.min_by(|a, b| a.health.total_cmp(&b.health)),
        Targeting::Nearest => in_range.min_by(|a, b| {
            origin
                .distance_squared(a.position)
                .total_cmp(&origin.distance_squared(b.position))
        }),
    };
    best.map(|c| c.entity)
}

/// Built turrets only: one under construction neither aims nor fires.
fn aim(
    mut towers: Query<
        (&Transform, &Range, &Targeting, &mut Aim),
        (With<Tower>, Without<Constructing>),
    >,
    enemies: Query<(Entity, &Transform, &Along, &Health), With<Enemy>>,
) {
    for (transform, range, targeting, mut aim) in &mut towers {
        let candidates = enemies
            .iter()
            .map(|(entity, ship, along, health)| Candidate {
                entity,
                position: ship.translation.truncate(),
                along: along.0,
                health: health.current,
            });
        aim.0 = pick_target(
            transform.translation.truncate(),
            range.0,
            *targeting,
            candidates,
        );
    }
}

fn fire(
    time: Res<Time>,
    mut commands: Commands,
    mut fired: MessageWriter<Fired>,
    mut towers: Query<
        (Entity, &Transform, &TowerType, &Tier, &Aim, &mut Gun),
        (With<Tower>, Without<Constructing>),
    >,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for (tower, tower_transform, kind, tier, aim, mut gun) in &mut towers {
        gun.reload.tick(time.delta());
        if !gun.reload.is_finished() {
            continue;
        }
        let Some((entity, target)) = aim.0.and_then(|e| enemies.get(e).ok().map(|t| (e, t))) else {
            continue;
        };

        let origin = tower_transform.translation.truncate();
        let dir = (target.translation.truncate() - origin).normalize_or(Vec2::Y);
        fired.write(Fired {
            weapon: Weapon::Turret(kind.0),
            shooter: tower,
            from: origin + dir * TOWER_SIZE[tier.0 as usize] * 0.5,
            dir,
        });
        gun.reload.reset();
        for _ in 0..gun.barrels {
            commands.spawn(shell_bundle(
                origin,
                entity,
                gun.damage,
                gun.ap,
                gun.projectile_speed,
                Weapon::Turret(kind.0),
            ));
        }
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
        if occupied || !treasury.try_spend(request.kind.stats(0).cost) {
            continue;
        }
        let tower = commands
            .spawn(tower_bundle(request.kind, transform.translation.truncate()))
            .id();
        commands.entity(request.spot).insert(Occupied(tower));
    }
}

fn finish_construction(
    time: Res<Time>,
    mut commands: Commands,
    mut sites: Query<(Entity, &TowerType, &Tier, &mut Constructing, &mut Sprite)>,
) {
    for (entity, kind, tier, mut constructing, mut sprite) in &mut sites {
        if constructing.0.tick(time.delta()).just_finished() {
            dress(&mut sprite, kind.0, tier.0, 1.0);
            commands.entity(entity).remove::<Constructing>();
        }
    }
}

fn draw_construction(mut gizmos: Gizmos, sites: Query<(&Transform, &Constructing)>) {
    for (transform, constructing) in &sites {
        gizmos.arc_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            std::f32::consts::TAU * constructing.0.fraction(),
            TILE_SIZE * 0.5,
            INK,
        );
    }
}

/// Greybox only; a drawn barrel is sized by `art`.
fn fit_barrels(
    towers: Query<(&Tier, &Children), (With<Tower>, Changed<Tier>)>,
    mut barrels: Query<&mut Sprite, With<Barrel>>,
) {
    for (tier, children) in &towers {
        for &child in children {
            if let Ok(mut sprite) = barrels.get_mut(child) {
                if sprite.image == Handle::default() {
                    sprite.custom_size = Some(Vec2::new(4.0, TOWER_SIZE[tier.0 as usize] * 0.55));
                }
            }
        }
    }
}

fn kick_barrels(
    mut fired: MessageReader<Fired>,
    towers: Query<&Children, With<Tower>>,
    mut barrels: Query<&mut Barrel>,
) {
    for shot in fired.read() {
        let Ok(children) = towers.get(shot.shooter) else {
            continue;
        };
        for &child in children {
            if let Ok(mut barrel) = barrels.get_mut(child) {
                barrel.recoil = RECOIL_DISTANCE;
            }
        }
    }
}

/// Turns each barrel towards its tower's aim and lets the recoil run out.
fn turn_barrels(
    time: Res<Time>,
    towers: Query<(&Transform, &Aim, &Children), With<Tower>>,
    mut barrels: Query<(&mut Transform, &mut Barrel), Without<Tower>>,
    enemies: Query<&Transform, (With<Enemy>, Without<Tower>, Without<Barrel>)>,
) {
    let step = time.delta_secs();
    for (tower, aim, children) in &towers {
        let wanted = aim.0.and_then(|e| enemies.get(e).ok()).map(|ship| {
            (ship.translation.truncate() - tower.translation.truncate()).normalize_or(Vec2::Y)
        });
        for &child in children {
            let Ok((mut transform, mut barrel)) = barrels.get_mut(child) else {
                continue;
            };
            let mut facing = (transform.rotation * Vec3::Y).truncate();
            if let Some(wanted) = wanted {
                facing = facing.rotate_towards(wanted, BARREL_TURN_RATE * step);
                transform.rotation = Quat::from_rotation_arc_2d(Vec2::Y, facing);
            }
            barrel.recoil = (barrel.recoil - RECOIL_DISTANCE / RECOIL_SECONDS * step).max(0.0);
            transform.translation = (-facing * barrel.recoil).extend(transform.translation.z);
        }
    }
}

/// Instant, so a turret never goes offline mid-wave for having been improved.
fn upgrade(
    mut requests: MessageReader<UpgradeTower>,
    mut treasury: ResMut<Treasury>,
    mut towers: Query<
        (
            &TowerType,
            &mut Tier,
            &mut Investment,
            &mut Range,
            &mut Gun,
            &mut Sprite,
        ),
        With<Tower>,
    >,
) {
    for request in requests.read() {
        let Ok((kind, mut tier, mut investment, mut range, mut gun, mut sprite)) =
            towers.get_mut(request.tower)
        else {
            continue;
        };
        let next = tier.0 + 1;
        if next >= TOWER_TIERS || !treasury.try_spend(kind.0.stats(next).cost) {
            continue;
        }
        tier.0 = next;
        investment.0 += kind.0.stats(next).cost;
        range.0 = kind.0.stats(next).range;
        *gun = gun_for(kind.0, next);
        let alpha = sprite.color.alpha();
        dress(&mut sprite, kind.0, next, alpha);
    }
}

fn sell(
    mut requests: MessageReader<SellTower>,
    mut commands: Commands,
    mut treasury: ResMut<Treasury>,
    towers: Query<&Investment, With<Tower>>,
    spots: Query<(Entity, &Occupied)>,
) {
    for request in requests.read() {
        let Ok(investment) = towers.get(request.tower) else {
            continue;
        };
        treasury.marks += (investment.0 as f32 * SELL_REFUND).round() as u32;
        commands.entity(request.tower).despawn();
        for (spot, occupied) in &spots {
            if occupied.0 == request.tower {
                commands.entity(spot).remove::<Occupied>();
            }
        }
    }
}

fn cycle_targeting(
    mut requests: MessageReader<CycleTargeting>,
    mut towers: Query<&mut Targeting, With<Tower>>,
) {
    for request in requests.read() {
        if let Ok(mut targeting) = towers.get_mut(request.tower) {
            *targeting = targeting.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fleet() -> Vec<Candidate> {
        let mut world = World::new();
        [
            (Vec2::new(10.0, 0.0), 300.0, 50.0),
            (Vec2::new(40.0, 0.0), 500.0, 20.0),
            (Vec2::new(-5.0, 0.0), 100.0, 90.0),
            (Vec2::new(900.0, 0.0), 900.0, 10.0),
        ]
        .into_iter()
        .map(|(position, along, health)| Candidate {
            entity: world.spawn_empty().id(),
            position,
            along,
            health,
        })
        .collect()
    }

    #[test]
    fn each_targeting_mode_picks_its_ship_and_ignores_the_one_out_of_range() {
        let pick = |mode| {
            let ships = fleet();
            let ids: Vec<Entity> = ships.iter().map(|c| c.entity).collect();
            let chosen = pick_target(Vec2::ZERO, 100.0, mode, ships).unwrap();
            ids.iter().position(|&e| e == chosen).unwrap()
        };
        assert_eq!(pick(Targeting::First), 1);
        assert_eq!(pick(Targeting::Strongest), 2);
        assert_eq!(pick(Targeting::Weakest), 1);
        assert_eq!(pick(Targeting::Nearest), 2);
    }

    #[test]
    fn nothing_in_range_means_no_target() {
        assert_eq!(
            pick_target(Vec2::new(5000.0, 0.0), 100.0, Targeting::First, fleet()),
            None
        );
    }
}
