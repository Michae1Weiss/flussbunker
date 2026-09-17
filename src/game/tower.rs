//! Shore batteries on build spots: construction, upgrades, selling, targeting, firing.
//! Sole writer of `Tower`, `TowerType`, `Tier`, `Investment`, `Targeting`, `Range`, `Gun`,
//! `Constructing`, `Occupied`. Rules: docs/design/turrets.md

use bevy::prelude::*;

use crate::balance::{
    TowerKind, SELL_REFUND, TILE_SIZE, TOWER_BUILD_SECONDS, TOWER_TIERS, Z_TOWER,
};
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::{Along, Enemy};
use crate::game::input::{BuildTower, CycleTargeting, SellTower, UpgradeTower};
use crate::game::map::BuildSpot;
use crate::game::projectile::shell_bundle;
use crate::game_plugin::Sim;
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
            fire.in_set(Sim::Targeting),
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
            Targeting::Strongest => "Staerkstes",
            Targeting::Weakest => "Schwaechstes",
            Targeting::Nearest => "Naechstes",
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

fn sprite_for(kind: TowerKind, tier: u8) -> Sprite {
    let stats = kind.stats(tier);
    Sprite {
        color: Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2]),
        custom_size: Some(Vec2::splat(TILE_SIZE * (0.5 + 0.1 * tier as f32))),
        ..default()
    }
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
        Range(stats.range),
        gun_for(kind, 0),
        Constructing(Timer::from_seconds(TOWER_BUILD_SECONDS, TimerMode::Once)),
        Sprite {
            color: sprite_for(kind, 0).color.with_alpha(0.4),
            ..sprite_for(kind, 0)
        },
        Transform::from_translation(position.extend(Z_TOWER)),
        DespawnOnExit(Screen::Gameplay),
    )
}

fn fire(
    time: Res<Time>,
    mut commands: Commands,
    mut towers: Query<
        (&Transform, &Range, &Targeting, &mut Gun),
        (With<Tower>, Without<Constructing>),
    >,
    enemies: Query<(Entity, &Transform, &Along, &Health), With<Enemy>>,
) {
    for (tower_transform, range, targeting, mut gun) in &mut towers {
        gun.reload.tick(time.delta());
        if !gun.reload.is_finished() {
            continue;
        }

        let origin = tower_transform.translation.truncate();
        let in_range = enemies.iter().filter(|(_, transform, _, _)| {
            origin.distance_squared(transform.translation.truncate()) <= range.0 * range.0
        });
        let target = match targeting {
            Targeting::First => in_range.max_by(|a, b| a.2 .0.total_cmp(&b.2 .0)),
            Targeting::Strongest => in_range.max_by(|a, b| a.3.current.total_cmp(&b.3.current)),
            Targeting::Weakest => in_range.min_by(|a, b| a.3.current.total_cmp(&b.3.current)),
            Targeting::Nearest => in_range.min_by(|a, b| {
                origin
                    .distance_squared(a.1.translation.truncate())
                    .total_cmp(&origin.distance_squared(b.1.translation.truncate()))
            }),
        };
        let Some((entity, _, _, _)) = target else {
            continue;
        };

        gun.reload.reset();
        for _ in 0..gun.barrels {
            commands.spawn(shell_bundle(
                origin,
                entity,
                gun.damage,
                gun.ap,
                gun.projectile_speed,
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
            *sprite = sprite_for(kind.0, tier.0);
            commands.entity(entity).remove::<Constructing>();
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
        *sprite = sprite_for(kind.0, next);
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
