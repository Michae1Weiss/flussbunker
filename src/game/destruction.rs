//! What happens when an enemy sinks: burn, explode, break or capsize by size class
//! and killing weapon; blasts that chain; debris that flies and lands; remains that sink.
//! Sole writer of `Wreck`, `Drifting`, `Blast`, `Debris`, `Crater`, `Sinking`, `Rng`.
//! Rules: docs/design/destruction.md

use bevy::prelude::*;

use crate::balance::{
    SizeClass, TowerKind, BLAST_DAMAGE, BLAST_RADIUS, CHAIN_DELAY, CRATER_CAP, DEBRIS_CAP,
    DEBRIS_COUNT, DEBRIS_GRAVITY, DEBRIS_TYPES, RIVER_HALF_WIDTH, SINK_SECONDS, TILE_SIZE,
    WRECK_CAP, WRECK_DRIFT_SPEED, Z_DEBRIS, Z_WRECK,
};
use crate::game::combat::{Damage, DamageKind, LastHitBy, Sunk, Weapon};
use crate::game::enemy::{Enemy, EnemyType, Hull};
use crate::game::map::RiverPath;
use crate::game::projectile::splash_bundle;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Rng>()
        .add_systems(
            FixedUpdate,
            (
                (drift, fly_debris, retire, sink).in_set(Sim::Movement),
                detonate.in_set(Sim::Targeting),
            ),
        )
        .add_observer(on_sunk);
}

/// xorshift64*, seeded once; the game never needs reproducible runs.
#[derive(Resource)]
pub struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }
}

impl Rng {
    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        ((self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32) / (1u64 << 24) as f32
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.unit()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Drifts to the bank and strands. No debris, no blast.
    Burn,
    /// Sinks in place; one piece of debris.
    Explode,
    /// Two halves sink; several pieces of debris and a blast.
    Break,
    /// Flips and sinks. Nothing else.
    Capsize,
}

/// The death-style matrix from docs/design/destruction.md.
pub fn style(class: SizeClass, weapon: Weapon) -> Style {
    use SizeClass::*;
    use Style::*;
    use Weapon::*;
    match (class, weapon) {
        (_, Ram) => Capsize,
        (_, Blast) => Break,
        (Light, Turret(TowerKind::Flak) | DeckGun) => Burn,
        (Light, Turret(TowerKind::Gun88) | Torpedo) => Explode,
        (Light, Turret(TowerKind::Heavy15)) => Break,
        (Medium, Turret(TowerKind::Flak) | DeckGun) => Burn,
        (Medium, Turret(TowerKind::Gun88)) => Explode,
        (Medium, Turret(TowerKind::Heavy15) | Torpedo) => Break,
        (Heavy, Torpedo) => Break,
        (Heavy, _) => Explode,
        (Capital, Torpedo) => Break,
        (Capital, _) => Explode,
        (Light | Medium, Enemy) => Explode,
    }
}

/// Cosmetic remains: a silhouette under the water or a stranded hull on the bank.
/// `born` is `Time::elapsed_secs`; the oldest stranded wrecks go first.
#[derive(Component)]
pub struct Wreck {
    born: f32,
}

/// Fades out and shrinks, then despawns. Never removed once added.
#[derive(Component)]
pub struct Sinking {
    timer: Timer,
    alpha: f32,
}

impl Sinking {
    fn start(alpha: f32) -> Self {
        Self {
            timer: Timer::from_seconds(SINK_SECONDS, TimerMode::Once),
            alpha,
        }
    }
}

const UNDERWATER: Color = Color::srgba(0.35, 0.42, 0.48, 0.8);
const STRANDED: Color = Color::srgba(0.72, 0.66, 0.60, 0.9);

/// Moves toward `to`, then stops.
#[derive(Component)]
pub struct Drifting {
    pub to: Vec2,
}

/// Damages every enemy within `radius` once the delay has passed. The delay is what
/// makes a chain of explosions readable.
#[derive(Component)]
pub struct Blast {
    pub radius: f32,
    pub damage: f32,
    pub delay: Timer,
}

/// A piece flying on a ballistic arc; `height` is drawn as scale.
#[derive(Component)]
pub struct Debris {
    /// Index into `DEBRIS_TYPES`.
    pub kind: usize,
    pub velocity: Vec2,
    pub height: f32,
    pub vertical_speed: f32,
    pub damage: f32,
    pub size: f32,
}

/// `born` as on `Wreck`.
#[derive(Component)]
pub struct Crater {
    born: f32,
}

fn on_sunk(
    sunk: On<Sunk>,
    mut commands: Commands,
    mut rng: ResMut<Rng>,
    time: Res<Time>,
    path: Res<RiverPath>,
    ships: Query<(&Transform, &Sprite, &Hull, &EnemyType, Option<&LastHitBy>), With<Enemy>>,
    airborne: Query<(), With<Debris>>,
) {
    let Ok((transform, sprite, hull, kind, last_hit)) = ships.get(sunk.entity) else {
        return;
    };
    let class = kind.0.stats().class;
    let weapon = last_hit.map_or(Weapon::Enemy, |h| h.0);
    let position = transform.translation.truncate();
    let heading = (transform.rotation * Vec3::Y).truncate();
    let size = sprite.custom_size.unwrap_or(Vec2::splat(hull.0));
    let born = time.elapsed_secs();
    let underwater = |position, rotation, size, image| {
        (
            wreck(position, rotation, size, image, UNDERWATER, born),
            Sinking::start(UNDERWATER.alpha()),
        )
    };

    match style(class, weapon) {
        Style::Burn => {
            let (along, _) = path.project(position);
            let side = path.tangent_at(along).perp() * rng.range(-1.0, 1.0).signum();
            let bank = path.point_at(along) + side * (RIVER_HALF_WIDTH + size.x * 0.5);
            commands.spawn((
                wreck(
                    position,
                    transform.rotation,
                    size,
                    sprite.image.clone(),
                    STRANDED,
                    born,
                ),
                Drifting { to: bank },
            ));
        }
        Style::Explode => {
            commands.spawn(underwater(
                position,
                transform.rotation,
                size,
                sprite.image.clone(),
            ));
            spawn_debris(
                &mut commands,
                &mut rng,
                position,
                1,
                airborne.iter().count(),
            );
            commands.spawn(splash_bundle(position, Color::srgb(1.0, 0.7, 0.3), size.y));
        }
        Style::Break => {
            for sign in [-1.0, 1.0] {
                let half = position + heading * sign * size.y * 0.3;
                let tilt = Quat::from_rotation_z(sign * 0.3) * transform.rotation;
                commands.spawn(underwater(
                    half,
                    tilt,
                    Vec2::new(size.x, size.y * 0.45),
                    sprite.image.clone(),
                ));
            }
            let (low, high) = DEBRIS_COUNT[class.index()];
            let count = rng.range(low as f32, high as f32 + 1.0) as usize;
            spawn_debris(
                &mut commands,
                &mut rng,
                position,
                count,
                airborne.iter().count(),
            );
            commands.spawn((
                Name::new("Blast"),
                DespawnOnExit(Screen::Gameplay),
                Blast {
                    radius: BLAST_RADIUS[class.index()],
                    damage: BLAST_DAMAGE[class.index()],
                    delay: Timer::from_seconds(CHAIN_DELAY, TimerMode::Once),
                },
                Transform::from_translation(position.extend(Z_DEBRIS)),
            ));
            commands.spawn(splash_bundle(
                position,
                Color::srgb(1.0, 0.55, 0.2),
                BLAST_RADIUS[class.index()] * 2.0,
            ));
        }
        Style::Capsize => {
            commands.spawn(underwater(
                position,
                transform.rotation * Quat::from_rotation_z(0.5),
                size,
                sprite.image.clone(),
            ));
        }
    }
}

fn wreck(
    position: Vec2,
    rotation: Quat,
    size: Vec2,
    image: Handle<Image>,
    color: Color,
    born: f32,
) -> impl Bundle {
    (
        Name::new("Wreck"),
        Wreck { born },
        DespawnOnExit(Screen::Gameplay),
        Sprite {
            image,
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position.extend(Z_WRECK)).with_rotation(rotation),
    )
}

fn spawn_debris(
    commands: &mut Commands,
    rng: &mut Rng,
    origin: Vec2,
    count: usize,
    airborne: usize,
) {
    let room = DEBRIS_CAP.saturating_sub(airborne);
    for i in 0..count.min(room) {
        // The first piece is the turret, the heaviest; the rest are drawn at random.
        let kind = if i == 0 {
            0
        } else {
            rng.range(1.0, DEBRIS_TYPES.len() as f32) as usize
        };
        let (damage, size) = DEBRIS_TYPES[kind];
        let direction = Vec2::from_angle(rng.range(0.0, std::f32::consts::TAU));
        let speed = rng.range(1.5, 4.0) * TILE_SIZE;
        commands.spawn((
            Name::new("Debris"),
            DespawnOnExit(Screen::Gameplay),
            Debris {
                kind,
                velocity: direction * speed,
                height: 0.0,
                vertical_speed: rng.range(2.0, 4.0) * TILE_SIZE,
                damage,
                size,
            },
            Sprite {
                color: Color::srgb(0.25, 0.22, 0.20),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_translation(origin.extend(Z_DEBRIS)),
        ));
    }
}

fn drift(
    time: Res<Time>,
    mut commands: Commands,
    mut wrecks: Query<(Entity, &mut Transform, &Drifting)>,
) {
    for (entity, mut transform, drifting) in &mut wrecks {
        let position = transform.translation.truncate();
        let to_bank = drifting.to - position;
        let step = WRECK_DRIFT_SPEED * time.delta_secs();
        if to_bank.length() <= step {
            transform.translation = drifting.to.extend(transform.translation.z);
            commands.entity(entity).remove::<Drifting>();
        } else {
            transform.translation += (to_bank.normalize() * step).extend(0.0);
        }
    }
}

/// Keeps the newest `WRECK_CAP` stranded wrecks and `CRATER_CAP` craters; the rest sink.
fn retire(
    mut commands: Commands,
    wrecks: Query<(Entity, &Wreck, &Sprite), Without<Sinking>>,
    craters: Query<(Entity, &Crater, &Sprite), Without<Sinking>>,
) {
    let mut sink_oldest = |mut remains: Vec<(Entity, f32, f32)>, cap: usize| {
        if remains.len() <= cap {
            return;
        }
        remains.sort_by(|a, b| a.1.total_cmp(&b.1));
        for &(entity, _, alpha) in &remains[..remains.len() - cap] {
            commands.entity(entity).insert(Sinking::start(alpha));
        }
    };
    sink_oldest(
        wrecks
            .iter()
            .map(|(e, w, s)| (e, w.born, s.color.alpha()))
            .collect(),
        WRECK_CAP,
    );
    sink_oldest(
        craters
            .iter()
            .map(|(e, c, s)| (e, c.born, s.color.alpha()))
            .collect(),
        CRATER_CAP,
    );
}

fn sink(
    time: Res<Time>,
    mut commands: Commands,
    mut remains: Query<(Entity, &mut Sinking, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut sinking, mut sprite, mut transform) in &mut remains {
        if sinking.timer.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let left = 1.0 - sinking.timer.fraction();
        sprite.color.set_alpha(sinking.alpha * left);
        transform.scale = Vec3::splat(0.85 + 0.15 * left);
    }
}

fn detonate(
    time: Res<Time>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut blasts: Query<(Entity, &Transform, &mut Blast)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (entity, transform, mut blast) in &mut blasts {
        if !blast.delay.tick(time.delta()).just_finished() {
            continue;
        }
        let centre = transform.translation.truncate();
        for (enemy, enemy_transform) in &enemies {
            if centre.distance_squared(enemy_transform.translation.truncate())
                <= blast.radius * blast.radius
            {
                damage.write(Damage {
                    target: enemy,
                    amount: blast.damage,
                    kind: DamageKind::Blast,
                    ap: false,
                    weapon: Weapon::Blast,
                });
            }
        }
        commands.entity(entity).despawn();
    }
}

/// Lands on a ship: damage. On water: a splash. On land: a crater.
fn fly_debris(
    time: Res<Time>,
    path: Res<RiverPath>,
    mut commands: Commands,
    mut damage: MessageWriter<Damage>,
    mut pieces: Query<(Entity, &mut Transform, &mut Debris)>,
    enemies: Query<(Entity, &Transform, &Hull), (With<Enemy>, Without<Debris>)>,
) {
    let step = time.delta_secs();
    for (entity, mut transform, mut piece) in &mut pieces {
        piece.vertical_speed -= DEBRIS_GRAVITY * step;
        piece.height += piece.vertical_speed * step;
        transform.translation += (piece.velocity * step).extend(0.0);
        transform.scale = Vec3::splat(1.0 + piece.height / (3.0 * TILE_SIZE));
        transform.rotate_z(3.0 * step);
        if piece.height > 0.0 {
            continue;
        }

        let landing = transform.translation.truncate();
        let hit = enemies
            .iter()
            .find(|(_, ship, hull)| ship.translation.truncate().distance(landing) <= hull.0 * 0.5);
        if let Some((enemy, _, _)) = hit {
            damage.write(Damage {
                target: enemy,
                amount: piece.damage,
                kind: DamageKind::Blast,
                ap: false,
                weapon: Weapon::Blast,
            });
            commands.spawn(splash_bundle(
                landing,
                Color::srgb(1.0, 0.8, 0.4),
                piece.size * 2.0,
            ));
        } else if path.contains(landing) {
            commands.spawn(splash_bundle(
                landing,
                Color::srgb(0.7, 0.85, 0.95),
                piece.size,
            ));
        } else {
            commands.spawn((
                Name::new("Crater"),
                Crater {
                    born: time.elapsed_secs(),
                },
                DespawnOnExit(Screen::Gameplay),
                Sprite {
                    color: Color::srgba(0.10, 0.12, 0.09, 0.5),
                    custom_size: Some(Vec2::splat(piece.size * 1.6)),
                    ..default()
                },
                Transform::from_translation(landing.extend(Z_WRECK)),
            ));
        }
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_matrix_matches_the_design_doc() {
        assert_eq!(
            style(SizeClass::Light, Weapon::Turret(TowerKind::Flak)),
            Style::Burn
        );
        assert_eq!(
            style(SizeClass::Light, Weapon::Turret(TowerKind::Heavy15)),
            Style::Break
        );
        assert_eq!(style(SizeClass::Light, Weapon::Torpedo), Style::Explode);
        assert_eq!(style(SizeClass::Light, Weapon::Ram), Style::Capsize);
        assert_eq!(style(SizeClass::Medium, Weapon::Torpedo), Style::Break);
        assert_eq!(
            style(SizeClass::Heavy, Weapon::Turret(TowerKind::Heavy15)),
            Style::Explode
        );
        assert_eq!(style(SizeClass::Heavy, Weapon::Torpedo), Style::Break);
        assert_eq!(style(SizeClass::Capital, Weapon::Torpedo), Style::Break);
        assert_eq!(style(SizeClass::Capital, Weapon::Blast), Style::Break);
    }

    #[test]
    fn remains_beyond_the_cap_sink_oldest_first() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        let wrecks: Vec<Entity> = (0..WRECK_CAP + 2)
            .map(|i| {
                world
                    .spawn((Wreck { born: i as f32 }, Sprite::default()))
                    .id()
            })
            .collect();
        world.run_system_once(retire).unwrap();
        let sinking: Vec<bool> = wrecks
            .iter()
            .map(|&e| world.entity(e).contains::<Sinking>())
            .collect();
        assert_eq!(sinking.iter().filter(|&&s| s).count(), 2);
        assert!(sinking[0] && sinking[1] && !sinking[2]);
    }

    #[test]
    fn rng_stays_in_unit_interval() {
        let mut rng = Rng::default();
        for _ in 0..10_000 {
            let x = rng.unit();
            assert!((0.0..1.0).contains(&x));
        }
    }
}
