//! The art layer: swaps greybox rectangles for sprites from `assets/` when the files
//! exist, applies the handwriting font, lays the painted map under the river. Gameplay
//! never knows whether a sprite is a rectangle or a drawing. Sole writer of `Art`,
//! `Sprite::image`, `TextFont::font`. Asset list and sizes: docs/ART.md

use std::collections::HashMap;

use bevy::prelude::*;

use crate::balance::{
    BoatModel, EnemyKind, TowerKind, BUILD_SPOT_RADIUS, MAP_HEIGHT_TILES, MAP_WIDTH_TILES,
    TILE_SIZE, TOWER_TIERS,
};
use crate::game::destruction::{Crater, Debris};
use crate::game::enemy::EnemyType;
use crate::game::map::{BuildSpot, BunkerSprite, RiverSegment};
use crate::game::projectile::{Lifetime, Shell, Torpedo};
use crate::game::tower::{Tier, Tower, TowerType};
use crate::game::uboat::{Model, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Art>()
        .add_systems(Startup, load)
        .add_systems(OnEnter(Screen::Gameplay), lay_background)
        .add_systems(Update, apply_font)
        .add_systems(
            Update,
            (
                dress_enemies,
                dress_towers,
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

/// Handles for every asset that exists on disk. Missing files stay greybox.
#[derive(Resource, Default)]
pub struct Art {
    images: HashMap<String, Handle<Image>>,
    font: Option<Handle<Font>>,
}

impl Art {
    pub fn image(&self, key: &str) -> Option<Handle<Image>> {
        self.images.get(key).cloned()
    }
}

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

fn tower_key(kind: TowerKind, tier: u8) -> String {
    let name = match kind {
        TowerKind::Flak => "flak",
        TowerKind::Gun88 => "gun88",
        TowerKind::Heavy15 => "heavy15",
    };
    format!("sprites/turrets/{name}_{tier}.png")
}

fn boat_key(model: BoatModel) -> &'static str {
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
    keys
}

#[cfg(not(target_arch = "wasm32"))]
fn exists(key: &str) -> bool {
    std::path::Path::new("assets").join(key).exists()
}

/// The web build cannot look before it loads; it trusts the list `tools/art_fit.py` keeps.
#[cfg(target_arch = "wasm32")]
fn exists(key: &str) -> bool {
    include_str!("../../assets/manifest.txt")
        .lines()
        .any(|line| line.trim() == key)
}

fn load(mut art: ResMut<Art>, assets: Res<AssetServer>) {
    for key in manifest() {
        if exists(&key) {
            art.images.insert(key.clone(), assets.load(key));
        }
    }
    if exists("fonts/pencil.ttf") {
        art.font = Some(assets.load("fonts/pencil.ttf"));
    }
    info!(
        "art layer: {} images, font: {}",
        art.images.len(),
        art.font.is_some()
    );
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

fn show_spots(
    wave: Option<Res<WaveState>>,
    mut spots: Query<&mut Visibility, (With<BuildSpot>, With<Sprite>)>,
) {
    let hidden = wave.is_some_and(|w| w.prologue);
    for mut visibility in &mut spots {
        *visibility = if hidden {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

/// The painted map covers the whole world under everything; the greybox river hides.
fn lay_background(art: Res<Art>, mut commands: Commands) {
    let Some(image) = art.image("map/background.png") else {
        return;
    };
    commands.spawn((
        Name::new("Background"),
        DespawnOnExit(Screen::Gameplay),
        Sprite {
            image,
            custom_size: Some(Vec2::new(
                MAP_WIDTH_TILES * TILE_SIZE,
                MAP_HEIGHT_TILES * TILE_SIZE,
            )),
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
