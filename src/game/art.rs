//! The art layer: swaps greybox rectangles for sprites from `assets/` when the files
//! exist, applies the handwriting font, lays the painted map under the river. Gameplay
//! never knows whether a sprite is a rectangle or a drawing. Sole writer of `Art`,
//! `Sprite::image`, `TextFont::font`, and of a drawn barrel's size and anchor.
//! Asset list and sizes: docs/ART.md

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::balance::{
    BoatModel, EnemyKind, TowerKind, BUILD_SPOT_RADIUS, MAP_BLEED_TILES, MAP_HEIGHT_TILES,
    MAP_WIDTH_TILES, TILE_SIZE, TOWER_SIZE, TOWER_TIERS,
};
use crate::game::destruction::{Crater, Debris};
use crate::game::enemy::EnemyType;
use crate::game::map::{BuildSpot, BunkerSprite, RiverSegment};
use crate::game::projectile::{Lifetime, Shell, Torpedo};
use crate::game::tower::{Barrel, Occupied, Tier, Tower, TowerType};
use crate::game::uboat::{Model, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Art>()
        .add_systems(OnEnter(Screen::Gameplay), lay_background)
        .add_systems(Update, apply_font)
        .add_systems(
            Update,
            (
                dress_enemies,
                dress_towers,
                dress_barrels,
                dress_boats,
                dress_projectiles,
                dress_debris,
                dress_craters,
                dress_splashes,
                dress_bunker,
                dress_spots,
                show_spots,
                hide_river,
            )
                .in_set(Frame::Present),
        );
}

/// Handles for every asset that exists on disk. Missing files stay greybox. Loaded when the
/// plugin is built, so it is ready before the first `OnEnter`.
#[derive(Resource)]
pub struct Art {
    images: HashMap<String, Handle<Image>>,
    font: Option<Handle<Font>>,
}

impl Art {
    pub fn image(&self, key: &str) -> Option<Handle<Image>> {
        self.images.get(key).cloned()
    }

    /// `prefix_0.png` .. `prefix_{n-1}.png`, or nothing unless every frame exists.
    pub fn frames(&self, prefix: &str, n: usize) -> Vec<Handle<Image>> {
        let frames: Vec<_> = (0..n)
            .filter_map(|i| self.image(&format!("{prefix}_{i}.png")))
            .collect();
        if frames.len() == n {
            frames
        } else {
            Vec::new()
        }
    }
}

/// A 9-slice paper drawing as the backmost child of a UI node, filling it; `None` until
/// `key` is drawn. Borders mirror `tools/art_fit.py` (8 px per world unit, a quarter each side).
pub fn paper_backing(art: &Art, key: &str) -> Option<impl Bundle> {
    let (w, h) = match key {
        "ui/note.png" => (48.0, 32.0),
        "ui/tag.png" => (32.0, 16.0),
        "ui/banner.png" => (96.0, 32.0),
        _ => return None,
    };
    let image = art.image(key)?;
    let (x, y) = (w * 8.0 * 0.25, h * 8.0 * 0.25);
    Some((
        Name::new("Paper"),
        ImageNode::new(image).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect {
                min_inset: Vec2::new(x, y),
                max_inset: Vec2::new(x, y),
            },
            max_corner_scale: 0.5,
            ..default()
        })),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ))
}

pub const ICONS: [&str; 12] = [
    "coin",
    "bunker",
    "torpedo",
    "hull",
    "dive",
    "surface",
    "first",
    "strongest",
    "weakest",
    "nearest",
    "upgrade",
    "sell",
];

fn enemy_key(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::MotorTorpedoBoat => "sprites/ships/mtb.png",
        EnemyKind::Corvette => "sprites/ships/corvette.png",
        EnemyKind::LandingCraft => "sprites/ships/landing.png",
        EnemyKind::Destroyer => "sprites/ships/destroyer.png",
        EnemyKind::Monitor => "sprites/ships/monitor.png",
        EnemyKind::GoldTransport => "sprites/ships/gold.png",
    }
}

fn kind_name(kind: TowerKind) -> &'static str {
    match kind {
        TowerKind::Flak => "flak",
        TowerKind::Gun88 => "gun88",
        TowerKind::Heavy15 => "heavy15",
    }
}

pub fn tower_key(kind: TowerKind, tier: u8) -> String {
    format!("sprites/turrets/{}_{tier}.png", kind_name(kind))
}

pub fn barrel_key(kind: TowerKind, tier: u8) -> String {
    format!("sprites/barrels/{}_{tier}.png", kind_name(kind))
}

pub fn boat_key(model: BoatModel) -> &'static str {
    match model {
        BoatModel::ViiA => "sprites/boats/viia.png",
        BoatModel::ViiC => "sprites/boats/viic.png",
        BoatModel::IxC => "sprites/boats/ixc.png",
    }
}

/// Every key the layer knows; docs/ART.md lists the same files.
fn manifest() -> Vec<String> {
    let mut keys: Vec<String> = [
        EnemyKind::MotorTorpedoBoat,
        EnemyKind::Corvette,
        EnemyKind::LandingCraft,
        EnemyKind::Destroyer,
        EnemyKind::Monitor,
        EnemyKind::GoldTransport,
    ]
    .iter()
    .map(|&k| enemy_key(k).to_string())
    .collect();
    for kind in TowerKind::ALL {
        for tier in 0..TOWER_TIERS {
            keys.push(tower_key(kind, tier));
            keys.push(barrel_key(kind, tier));
        }
    }
    for model in [BoatModel::ViiA, BoatModel::ViiC, BoatModel::IxC] {
        keys.push(boat_key(model).to_string());
    }
    keys.extend(
        [
            "sprites/torpedo.png",
            "sprites/shell.png",
            "sprites/crater.png",
            "sprites/splash.png",
            "sprites/bunker.png",
            "sprites/spot.png",
            "map/background.png",
        ]
        .map(String::from),
    );
    for i in 0..6 {
        keys.push(format!("sprites/debris_{i}.png"));
    }
    keys.extend((0..2).map(|i| format!("fx/flash_{i}.png")));
    keys.extend((0..4).map(|i| format!("fx/explosion_{i}.png")));
    keys.extend(
        [
            "fx/smoke.png",
            "ui/note.png",
            "ui/tag.png",
            "ui/banner.png",
            "ui/disc.png",
            "ui/stroke.png",
        ]
        .map(String::from),
    );
    keys.extend(ICONS.map(|name| format!("ui/icons/{name}.png")));
    keys
}

/// Whether `assets/<key>` ships with the game; audio uses it too.
#[cfg(not(target_arch = "wasm32"))]
pub fn exists(key: &str) -> bool {
    std::path::Path::new("assets").join(key).exists()
}

/// The web build cannot look before it loads; it trusts the list `tools/art_fit.py` keeps.
#[cfg(target_arch = "wasm32")]
pub fn exists(key: &str) -> bool {
    include_str!("../../assets/manifest.txt")
        .lines()
        .any(|line| line.trim() == key)
}

impl FromWorld for Art {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let images: HashMap<_, _> = manifest()
            .into_iter()
            .filter(|key| exists(key))
            .map(|key| (key.clone(), assets.load(key)))
            .collect();
        let font = exists("fonts/pencil.ttf").then(|| assets.load("fonts/pencil.ttf"));
        info!(
            "art layer: {} images, font: {}",
            images.len(),
            font.is_some()
        );
        Self { images, font }
    }
}

fn dress(sprite: &mut Sprite, image: Option<Handle<Image>>) {
    if let Some(image) = image {
        sprite.image = image;
        sprite.color = Color::WHITE.with_alpha(sprite.color.alpha());
    }
}

fn dress_enemies(art: Res<Art>, mut ships: Query<(&EnemyType, &mut Sprite), Added<EnemyType>>) {
    for (kind, mut sprite) in &mut ships {
        dress(&mut sprite, art.image(enemy_key(kind.0)));
    }
}

fn dress_towers(
    art: Res<Art>,
    mut towers: Query<(&TowerType, &Tier, &mut Sprite), Or<(Added<Tower>, Changed<Tier>)>>,
) {
    for (kind, tier, mut sprite) in &mut towers {
        dress(&mut sprite, art.image(&tower_key(kind.0, tier.0)));
    }
}

/// A drawn barrel shares its emplacement's square and turns around the centre.
fn dress_barrels(
    art: Res<Art>,
    towers: Query<(&TowerType, &Tier, &Children), Or<(Added<Tower>, Changed<Tier>)>>,
    mut barrels: Query<(&mut Sprite, &mut Anchor), With<Barrel>>,
) {
    for (kind, tier, children) in &towers {
        let Some(image) = art.image(&barrel_key(kind.0, tier.0)) else {
            continue;
        };
        for &child in children {
            if let Ok((mut sprite, mut anchor)) = barrels.get_mut(child) {
                sprite.image = image.clone();
                sprite.color = Color::WHITE;
                sprite.custom_size = Some(Vec2::splat(TOWER_SIZE[tier.0 as usize]));
                *anchor = Anchor::CENTER;
            }
        }
    }
}

fn dress_boats(
    art: Res<Art>,
    mut boats: Query<(&Model, &mut Sprite), Or<(Added<UBoat>, Changed<Model>)>>,
) {
    for (model, mut sprite) in &mut boats {
        dress(&mut sprite, art.image(boat_key(model.0)));
    }
}

fn dress_projectiles(
    art: Res<Art>,
    mut torpedoes: Query<&mut Sprite, Added<Torpedo>>,
    mut shells: Query<&mut Sprite, (Added<Shell>, Without<Torpedo>)>,
) {
    for mut sprite in &mut torpedoes {
        dress(&mut sprite, art.image("sprites/torpedo.png"));
    }
    for mut sprite in &mut shells {
        dress(&mut sprite, art.image("sprites/shell.png"));
    }
}

fn dress_debris(art: Res<Art>, mut pieces: Query<(&Debris, &mut Sprite), Added<Debris>>) {
    for (piece, mut sprite) in &mut pieces {
        dress(
            &mut sprite,
            art.image(&format!("sprites/debris_{}.png", piece.kind)),
        );
    }
}

fn dress_craters(art: Res<Art>, mut craters: Query<&mut Sprite, Added<Crater>>) {
    for mut sprite in &mut craters {
        dress(&mut sprite, art.image("sprites/crater.png"));
    }
}

fn dress_splashes(art: Res<Art>, mut splashes: Query<&mut Sprite, Added<Lifetime>>) {
    for mut sprite in &mut splashes {
        dress(&mut sprite, art.image("sprites/splash.png"));
    }
}

fn dress_bunker(art: Res<Art>, mut bunkers: Query<&mut Sprite, Added<BunkerSprite>>) {
    for mut sprite in &mut bunkers {
        dress(&mut sprite, art.image("sprites/bunker.png"));
    }
}

fn dress_spots(art: Res<Art>, mut commands: Commands, spots: Query<Entity, Added<BuildSpot>>) {
    let Some(image) = art.image("sprites/spot.png") else {
        return;
    };
    for spot in &spots {
        commands.entity(spot).insert((
            Sprite {
                image: image.clone(),
                custom_size: Some(Vec2::splat(BUILD_SPOT_RADIUS * 2.0)),
                ..default()
            },
            Visibility::Hidden,
        ));
    }
}

/// Hidden in the prologue and under a turret.
fn show_spots(
    wave: Option<Res<WaveState>>,
    mut spots: Query<(&mut Visibility, Has<Occupied>), (With<BuildSpot>, With<Sprite>)>,
) {
    let prologue = wave.is_some_and(|w| w.prologue);
    for (mut visibility, occupied) in &mut spots {
        *visibility = if prologue || occupied {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// The painted map covers the world plus its bleed under everything; the greybox river hides.
fn lay_background(art: Res<Art>, mut commands: Commands) {
    let Some(image) = art.image("map/background.png") else {
        return;
    };
    commands.spawn((
        Name::new("Background"),
        DespawnOnExit(Screen::Gameplay),
        Sprite {
            image,
            custom_size: Some(
                (Vec2::new(MAP_WIDTH_TILES, MAP_HEIGHT_TILES) + 2.0 * MAP_BLEED_TILES) * TILE_SIZE,
            ),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
    ));
}

fn hide_river(art: Res<Art>, mut segments: Query<&mut Visibility, Added<RiverSegment>>) {
    if art.image("map/background.png").is_none() {
        return;
    }
    for mut visibility in &mut segments {
        *visibility = Visibility::Hidden;
    }
}

fn apply_font(art: Res<Art>, mut fonts: Query<&mut TextFont, Added<TextFont>>) {
    let Some(font) = art.font.as_ref() else {
        return;
    };
    for mut text_font in &mut fonts {
        text_font.font = FontSource::Handle(font.clone());
    }
}
