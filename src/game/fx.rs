//! Presentation only: muzzle flashes, hit sparks, explosions and smoke, read from `Fired`,
//! `Damage`, `Sunk` and `TorpedoLost`. Drawn frames come from `art`; without them a soft
//! disc stands in. Sole writer of `Burst`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::balance::{TowerKind, BLAST_RADIUS, Z_DEBRIS};
use crate::game::art::Art;
use crate::game::combat::{Damage, DamageKind, Fired, LastHitBy, Sunk, Weapon};
use crate::game::destruction::{style, Style};
use crate::game::enemy::{Enemy, EnemyType, Hull};
use crate::game::map::RiverPath;
use crate::game::projectile::TorpedoLost;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, make_disc)
        .add_systems(
            Update,
            (flash_muzzles, spark_hits, burst_banks, play)
                .chain()
                .in_set(Frame::Present),
        )
        .add_observer(explode_sunk);
}

/// A soft white disc: the stand-in for every effect that is not drawn yet.
#[derive(Resource)]
struct Disc(Handle<Image>);

/// Plays `frames` over the timer; with no frames the disc grows and fades instead.
/// `grow` is the extra scale reached at the end, `drift` world units per second.
#[derive(Component)]
struct Burst {
    timer: Timer,
    frames: Vec<Handle<Image>>,
    alpha: f32,
    grow: f32,
    drift: Vec2,
}

const FLAME: Color = Color::srgb(1.0, 0.82, 0.45);
const WATER: Color = Color::srgb(0.92, 0.96, 1.0);
const SMOKE: Color = Color::srgba(0.35, 0.33, 0.31, 0.7);
const DIRT: Color = Color::srgba(0.45, 0.38, 0.30, 0.7);

fn make_disc(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    const SIDE: u32 = 64;
    let mut data = Vec::with_capacity((SIDE * SIDE * 4) as usize);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let d = Vec2::new(x as f32 + 0.5, y as f32 + 0.5)
                .distance(Vec2::splat(SIDE as f32 / 2.0))
                / (SIDE as f32 / 2.0);
            let alpha = ((1.0 - d) * 3.0).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    let image = Image::new(
        Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    commands.insert_resource(Disc(images.add(image)));
}

/// What one effect looks like; `spawn` turns it into an entity.
struct Effect {
    frames: Vec<Handle<Image>>,
    stand_in: Color,
    size: f32,
    seconds: f32,
    grow: f32,
    drift: Vec2,
    rotation: Quat,
}

impl Effect {
    fn new(frames: Vec<Handle<Image>>, stand_in: Color, size: f32, seconds: f32) -> Self {
        Self {
            frames,
            stand_in,
            size,
            seconds,
            grow: 0.3,
            drift: Vec2::ZERO,
            rotation: Quat::IDENTITY,
        }
    }

    fn spawn(self, commands: &mut Commands, disc: &Disc, at: Vec2, z: f32) {
        let (image, color) = match self.frames.first() {
            Some(first) => (first.clone(), Color::WHITE),
            None => (disc.0.clone(), self.stand_in),
        };
        commands.spawn((
            Name::new("Effect"),
            Burst {
                timer: Timer::from_seconds(self.seconds, TimerMode::Once),
                frames: self.frames,
                alpha: color.alpha(),
                grow: self.grow,
                drift: self.drift,
            },
            Sprite {
                image,
                color,
                custom_size: Some(Vec2::splat(self.size)),
                ..default()
            },
            Transform::from_translation(at.extend(z)).with_rotation(self.rotation),
            DespawnOnExit(Screen::Gameplay),
        ));
    }
}

fn explosion(art: &Art, size: f32) -> Effect {
    Effect::new(art.frames("fx/explosion", 4), FLAME, size, 0.32)
}

fn smoke(art: &Art, size: f32, color: Color) -> Effect {
    let frames = art.image("fx/smoke.png").into_iter().collect();
    let mut effect = Effect::new(frames, color, size, 1.4);
    effect.grow = 0.8;
    effect.drift = Vec2::new(6.0, 10.0);
    effect
}

fn muzzle_size(weapon: Weapon) -> f32 {
    match weapon {
        Weapon::Turret(TowerKind::Flak) => 10.0,
        Weapon::Turret(TowerKind::Gun88) => 14.0,
        Weapon::Turret(TowerKind::Heavy15) => 20.0,
        Weapon::DeckGun => 12.0,
        _ => 14.0,
    }
}

fn flash_muzzles(
    mut fired: MessageReader<Fired>,
    mut commands: Commands,
    art: Res<Art>,
    disc: Res<Disc>,
) {
    for shot in fired.read() {
        if shot.weapon == Weapon::Torpedo {
            let mut bubbles = Effect::new(Vec::new(), WATER, 12.0, 0.4);
            bubbles.grow = 0.6;
            bubbles.spawn(&mut commands, &disc, shot.from, Z_DEBRIS);
            continue;
        }
        let size = muzzle_size(shot.weapon);
        let mut flash = Effect::new(art.frames("fx/flash", 2), FLAME, size, 0.09);
        flash.rotation = Quat::from_rotation_arc_2d(Vec2::Y, shot.dir);
        flash.spawn(
            &mut commands,
            &disc,
            shot.from + shot.dir * size * 0.35,
            Z_DEBRIS,
        );
    }
}

/// Shells spark on the hull; a torpedo hit is a proper explosion.
fn spark_hits(
    mut hits: MessageReader<Damage>,
    mut commands: Commands,
    art: Res<Art>,
    disc: Res<Disc>,
    targets: Query<&Transform>,
) {
    for hit in hits.read() {
        let Ok(target) = targets.get(hit.target) else {
            continue;
        };
        let at = target.translation.truncate();
        match hit.kind {
            DamageKind::Shell => {
                Effect::new(art.frames("fx/flash", 2), FLAME, 7.0, 0.07).spawn(
                    &mut commands,
                    &disc,
                    at,
                    Z_DEBRIS,
                );
            }
            DamageKind::Torpedo => {
                explosion(&art, 44.0).spawn(&mut commands, &disc, at, Z_DEBRIS);
                smoke(&art, 30.0, SMOKE).spawn(&mut commands, &disc, at, Z_DEBRIS - 1.0);
            }
            DamageKind::Ram | DamageKind::Blast => {}
        }
    }
}

/// On the bank a torpedo explodes into earth; in open water it just stops.
fn burst_banks(
    mut lost: MessageReader<TorpedoLost>,
    mut commands: Commands,
    art: Res<Art>,
    disc: Res<Disc>,
    path: Res<RiverPath>,
) {
    for lost in lost.read() {
        if path.contains(lost.at) {
            Effect::new(Vec::new(), WATER, 10.0, 0.4).spawn(
                &mut commands,
                &disc,
                lost.at,
                Z_DEBRIS,
            );
        } else {
            explosion(&art, 34.0).spawn(&mut commands, &disc, lost.at, Z_DEBRIS);
            smoke(&art, 26.0, DIRT).spawn(&mut commands, &disc, lost.at, Z_DEBRIS - 1.0);
        }
    }
}

fn explode_sunk(
    sunk: On<Sunk>,
    mut commands: Commands,
    art: Res<Art>,
    disc: Res<Disc>,
    ships: Query<(&Transform, &Hull, &EnemyType, Option<&LastHitBy>), With<Enemy>>,
) {
    let Ok((transform, hull, kind, last_hit)) = ships.get(sunk.entity) else {
        return;
    };
    let at = transform.translation.truncate();
    let class = kind.0.stats().class;
    let weapon = last_hit.map_or(Weapon::Enemy, |h| h.0);
    match style(class, weapon) {
        Style::Burn => {
            explosion(&art, hull.0 * 0.7).spawn(&mut commands, &disc, at, Z_DEBRIS);
            smoke(&art, hull.0 * 0.8, SMOKE).spawn(&mut commands, &disc, at, Z_DEBRIS - 1.0);
        }
        Style::Explode => {
            explosion(&art, hull.0 * 1.3).spawn(&mut commands, &disc, at, Z_DEBRIS);
            smoke(&art, hull.0, SMOKE).spawn(&mut commands, &disc, at, Z_DEBRIS - 1.0);
        }
        Style::Break => {
            explosion(&art, BLAST_RADIUS[class.index()] * 2.0).spawn(
                &mut commands,
                &disc,
                at,
                Z_DEBRIS,
            );
            smoke(&art, hull.0 * 1.4, SMOKE).spawn(&mut commands, &disc, at, Z_DEBRIS - 1.0);
        }
        Style::Capsize => {
            let mut spray = Effect::new(Vec::new(), WATER, hull.0 * 0.8, 0.5);
            spray.grow = 0.6;
            spray.spawn(&mut commands, &disc, at, Z_DEBRIS);
        }
    }
}

fn play(
    time: Res<Time>,
    mut commands: Commands,
    mut bursts: Query<(Entity, &mut Burst, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut burst, mut sprite, mut transform) in &mut bursts {
        if burst.timer.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let t = burst.timer.fraction();
        let fade = match burst.frames.len() {
            0 | 1 => 1.0 - t,
            n => {
                sprite.image = burst.frames[((t * n as f32) as usize).min(n - 1)].clone();
                ((1.0 - t) / 0.3).min(1.0)
            }
        };
        sprite.color.set_alpha(burst.alpha * fade);
        transform.scale = Vec3::splat(1.0 + burst.grow * t);
        transform.translation += (burst.drift * time.delta_secs()).extend(0.0);
    }
}
