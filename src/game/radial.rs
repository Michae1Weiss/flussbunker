//! The radial menu around a build spot or turret: entries, hit-testing, drawing. Each entry
//! is a paper disc with a picture (the emplacement or boat it buys, a coin, the targeting
//! mode) and its price below; text only where nothing is drawn yet. Opens and closes on
//! messages from `input`, which also decides what a click means.
//! Sole writer of `Menu` and of the entries' visuals. Rules: docs/design/turrets.md#radial-menu

use bevy::prelude::*;

use crate::balance::{
    TowerKind, RADIAL_ENTRY_RADIUS, RADIAL_RADIUS, SELL_REFUND, TOWER_TIERS, Z_UI,
};
use crate::game::art::{barrel_key, boat_key, tower_key, Art};
use crate::game::economy::Treasury;
use crate::game::input::{Hovered, SellHold};
use crate::game::palette::{FAINT, INK, NOTE, PENCIL, RED};
use crate::game::stroke::Strokes;
use crate::game::tower::{Investment, Targeting, Tier, TowerType};
use crate::game::uboat::Flotilla;
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Menu>()
        .add_message::<OpenMenu>()
        .add_message::<CloseMenu>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            Update,
            (
                (close, open).chain().in_set(Frame::Input),
                (refresh, draw).chain().in_set(Frame::Present),
            ),
        );
}

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Spot(Entity),
    Turret { spot: Entity, tower: Entity },
    Boat(Entity),
}

#[derive(Message)]
pub struct OpenMenu {
    pub target: Target,
    /// World position the entries are arranged around.
    pub anchor: Vec2,
}

#[derive(Message)]
pub struct CloseMenu;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Build(TowerKind),
    Upgrade,
    Sell,
    Targeting,
    UpgradeBoat,
}

pub struct Entry {
    pub action: Action,
    /// World offset from the anchor.
    pub offset: Vec2,
    pub enabled: bool,
    /// The entry's own root, and its picture, the picture's barrel, and its text.
    node: Entity,
    icon: Entity,
    barrel: Entity,
    label: Entity,
}

pub struct Open {
    pub target: Target,
    pub anchor: Vec2,
    pub entries: Vec<Entry>,
    root: Entity,
}

#[derive(Resource, Default)]
pub struct Menu {
    pub open: Option<Open>,
}

impl Menu {
    /// The enabled entry under `world`, if any.
    pub fn hit(&self, world: Vec2) -> Option<&Entry> {
        self.hit_any(world).filter(|e| e.enabled)
    }

    /// The entry under `world`, enabled or not.
    pub fn hit_any(&self, world: Vec2) -> Option<&Entry> {
        let open = self.open.as_ref()?;
        open.entries
            .iter()
            .find(|e| (open.anchor + e.offset).distance(world) <= RADIAL_ENTRY_RADIUS)
    }
}

fn reset(mut menu: ResMut<Menu>) {
    menu.open = None;
}

fn close(mut requests: MessageReader<CloseMenu>, mut menu: ResMut<Menu>, mut commands: Commands) {
    if requests.read().count() == 0 {
        return;
    }
    if let Some(open) = menu.open.take() {
        commands.entity(open.root).despawn();
    }
}

fn open(
    mut requests: MessageReader<OpenMenu>,
    mut menu: ResMut<Menu>,
    mut commands: Commands,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(request) = requests.read().last() else {
        return;
    };
    if let Some(open) = menu.open.take() {
        commands.entity(open.root).despawn();
    }

    let root = commands
        .spawn((
            Name::new("Radial menu"),
            DespawnOnExit(Screen::Gameplay),
            Transform::from_translation(request.anchor.extend(Z_UI)),
            Visibility::default(),
        ))
        .id();

    let actions: Vec<Action> = match request.target {
        Target::Spot(_) => TowerKind::ALL.iter().map(|&k| Action::Build(k)).collect(),
        Target::Turret { .. } => vec![Action::Upgrade, Action::Sell, Action::Targeting],
        Target::Boat(_) => vec![Action::UpgradeBoat],
    };
    let entries = actions
        .iter()
        .enumerate()
        .map(|(i, &action)| {
            let angle = std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 3.0;
            let offset = Vec2::from_angle(angle) * RADIAL_RADIUS;
            let node = commands
                .spawn((
                    Name::new("Radial entry"),
                    Transform::from_translation(offset.extend(1.0)),
                    Visibility::default(),
                    ChildOf(root),
                ))
                .id();
            match art.image("ui/disc.png") {
                Some(image) => commands.spawn((
                    Sprite {
                        image,
                        custom_size: Some(Vec2::splat(RADIAL_ENTRY_RADIUS * 2.2)),
                        ..default()
                    },
                    ChildOf(node),
                )),
                None => commands.spawn((
                    Mesh2d(meshes.add(Circle::new(RADIAL_ENTRY_RADIUS))),
                    MeshMaterial2d(materials.add(NOTE)),
                    ChildOf(node),
                )),
            };
            let icon = commands
                .spawn((
                    Sprite::default(),
                    Visibility::Hidden,
                    Transform::from_translation(Vec3::new(0.0, ICON_LIFT, 0.1)),
                    ChildOf(node),
                ))
                .id();
            let barrel = commands
                .spawn((
                    Sprite::default(),
                    Visibility::Hidden,
                    Transform::from_translation(Vec3::Z * 0.1),
                    ChildOf(icon),
                ))
                .id();
            let label = commands
                .spawn((
                    Text2d::new(String::new()),
                    TextFont {
                        font_size: FontSize::Px(17.0),
                        ..default()
                    },
                    TextLayout::new(Justify::Center, bevy::text::LineBreak::WordBoundary),
                    TextColor(INK),
                    Transform::from_translation(Vec3::Z * 0.2),
                    ChildOf(node),
                ))
                .id();
            Entry {
                action,
                offset,
                enabled: false,
                node,
                icon,
                barrel,
                label,
            }
        })
        .collect();

    menu.open = Some(Open {
        target: request.target,
        anchor: request.anchor,
        entries,
        root,
    });
}

/// World units the picture sits above the entry's centre, making room for the price.
const ICON_LIFT: f32 = 6.0;

/// What an entry shows: a picture (with an optional barrel on top), the text beside it
/// when the picture exists, the text alone when it does not, and whether it can be bought.
struct Face {
    picture: Option<(Handle<Image>, Vec2)>,
    barrel: Option<Handle<Image>>,
    short: String,
    long: String,
    enabled: bool,
}

fn emplacement(
    art: &Art,
    kind: TowerKind,
    tier: u8,
) -> (Option<(Handle<Image>, Vec2)>, Option<Handle<Image>>) {
    let size = Vec2::splat(RADIAL_ENTRY_RADIUS * 1.1);
    (
        art.image(&tower_key(kind, tier)).map(|i| (i, size)),
        art.image(&barrel_key(kind, tier)),
    )
}

fn icon(art: &Art, name: &str) -> Option<(Handle<Image>, Vec2)> {
    art.image(&format!("ui/icons/{name}.png"))
        .map(|i| (i, Vec2::splat(RADIAL_ENTRY_RADIUS * 1.0)))
}

fn targeting_icon(targeting: Targeting) -> &'static str {
    match targeting {
        Targeting::First => "first",
        Targeting::Strongest => "strongest",
        Targeting::Weakest => "weakest",
        Targeting::Nearest => "nearest",
    }
}

/// Pictures, prices and affordability follow the treasury and the turret every frame; the
/// hovered entry grows a little.
fn refresh(
    mut menu: ResMut<Menu>,
    art: Res<Art>,
    treasury: Res<Treasury>,
    flotilla: Res<Flotilla>,
    hovered: Res<Hovered>,
    wave: Option<Res<WaveState>>,
    towers: Query<(&TowerType, &Tier, &Investment, &Targeting)>,
    mut labels: Query<(&mut Text2d, &mut TextColor, &mut Transform)>,
    mut sprites: Query<(&mut Sprite, &mut Visibility), Without<Text2d>>,
    mut nodes: Query<&mut Transform, (Without<Text2d>, Without<Sprite>)>,
) {
    let Some(open) = menu.open.as_mut() else {
        return;
    };
    let turret = match open.target {
        Target::Turret { tower, .. } => towers.get(tower).ok(),
        Target::Spot(_) | Target::Boat(_) => None,
    };
    let wave_index = wave.map_or(usize::MAX, |w| w.index);

    for entry in &mut open.entries {
        let face = match (entry.action, turret) {
            (Action::Build(kind), _) => {
                let stats = kind.stats(0);
                let (picture, barrel) = emplacement(&art, kind, 0);
                Face {
                    picture,
                    barrel,
                    short: format!("{} RM", stats.cost),
                    long: format!("{}\n{} RM", stats.name, stats.cost),
                    enabled: treasury.marks >= stats.cost,
                }
            }
            (Action::Upgrade, Some((kind, tier, _, _))) => {
                let next = tier.0 + 1;
                let shown = next.min(TOWER_TIERS - 1);
                let (picture, barrel) = emplacement(&art, kind.0, shown);
                let (short, long, enabled) = if next < TOWER_TIERS {
                    let cost = kind.0.stats(next).cost;
                    (
                        format!("{cost} RM"),
                        format!("Ausbau\n{cost} RM"),
                        treasury.marks >= cost,
                    )
                } else {
                    ("Max".into(), "Max".into(), false)
                };
                Face {
                    picture,
                    barrel,
                    short,
                    long,
                    enabled,
                }
            }
            (Action::Sell, Some((_, _, investment, _))) => {
                let refund = (investment.0 as f32 * SELL_REFUND).round();
                Face {
                    picture: icon(&art, "sell").or_else(|| icon(&art, "coin")),
                    barrel: None,
                    short: format!("+{refund} RM"),
                    long: format!("Verkaufen\n+{refund} RM"),
                    enabled: true,
                }
            }
            (Action::Targeting, Some((_, _, _, targeting))) => Face {
                picture: icon(&art, targeting_icon(*targeting)),
                barrel: None,
                short: targeting.label().into(),
                long: format!("Ziel:\n{}", targeting.label()),
                enabled: true,
            },
            (Action::UpgradeBoat, _) => match flotilla.model.next() {
                Some(next) => {
                    let stats = next.stats();
                    let unlocked = wave_index >= stats.unlock_wave;
                    let short = if unlocked {
                        format!("{} RM", stats.price)
                    } else {
                        format!("ab Welle {}", stats.unlock_wave + 1)
                    };
                    Face {
                        picture: art.image(boat_key(next)).map(|i| {
                            (
                                i,
                                Vec2::new(RADIAL_ENTRY_RADIUS * 0.45, RADIAL_ENTRY_RADIUS * 1.2),
                            )
                        }),
                        barrel: None,
                        long: format!("{}\n{short}", stats.name),
                        short,
                        enabled: unlocked && treasury.marks >= stats.price,
                    }
                }
                None => Face {
                    picture: None,
                    barrel: None,
                    short: String::new(),
                    long: "Max".into(),
                    enabled: false,
                },
            },
            (_, None) => Face {
                picture: None,
                barrel: None,
                short: String::new(),
                long: String::new(),
                enabled: false,
            },
        };
        entry.enabled = face.enabled;
        let tint = if face.enabled {
            Color::WHITE
        } else {
            Color::WHITE.with_alpha(0.4)
        };
        if let Ok((mut sprite, mut visibility)) = sprites.get_mut(entry.icon) {
            match &face.picture {
                Some((image, size)) => {
                    sprite.image = image.clone();
                    sprite.custom_size = Some(*size);
                    sprite.color = tint;
                    *visibility = Visibility::Inherited;
                }
                None => *visibility = Visibility::Hidden,
            }
        }
        if let Ok((mut sprite, mut visibility)) = sprites.get_mut(entry.barrel) {
            match (&face.barrel, &face.picture) {
                (Some(image), Some((_, size))) => {
                    sprite.image = image.clone();
                    sprite.custom_size = Some(*size);
                    sprite.color = tint;
                    *visibility = Visibility::Inherited;
                }
                _ => *visibility = Visibility::Hidden,
            }
        }
        if let Ok((mut label, mut color, mut transform)) = labels.get_mut(entry.label) {
            let (text, y, size) = if face.picture.is_some() {
                (face.short, -RADIAL_ENTRY_RADIUS * 0.62, 14.0)
            } else {
                (face.long, 0.0, 17.0)
            };
            if label.0 != text {
                label.0 = text;
            }
            transform.translation.y = y;
            transform.scale = Vec3::splat(size / 17.0);
            color.0 = if face.enabled { INK } else { PENCIL };
        }
        if let Ok(mut transform) = nodes.get_mut(entry.node) {
            let grow = if hovered.0 == Some(entry.action) {
                1.1
            } else {
                1.0
            };
            transform.scale = Vec3::splat(grow);
        }
    }
}

/// A pencil ring marks state only on the greybox disc; the drawn disc has its own edge, and
/// its faded picture and hover growth say the same. The sell ring fills outside the disc.
fn draw(
    mut strokes: ResMut<Strokes>,
    art: Res<Art>,
    menu: Res<Menu>,
    hold: Res<SellHold>,
    hovered: Res<Hovered>,
    towers: Query<(&TowerType, &Tier)>,
) {
    let Some(open) = menu.open.as_ref() else {
        return;
    };
    let hovered = hovered.0;
    let drawn = art.image("ui/disc.png").is_some();

    for entry in &open.entries {
        let centre = open.anchor + entry.offset;
        let is_hovered = hovered == Some(entry.action);
        let color = match (entry.enabled, is_hovered) {
            (false, _) => FAINT,
            (true, true) => RED,
            (true, false) => INK,
        };
        if !drawn {
            strokes.circle(centre, RADIAL_ENTRY_RADIUS, color);
        }

        if entry.action == Action::Sell && hold.progress > 0.0 {
            strokes.arc(
                centre,
                RADIAL_ENTRY_RADIUS * 1.3,
                std::f32::consts::TAU * hold.progress,
                RED,
            );
        }

        if !is_hovered {
            continue;
        }
        let range = match (entry.action, open.target) {
            (Action::Build(kind), _) => Some(kind.stats(0).range),
            (Action::Upgrade, Target::Turret { tower, .. }) => towers
                .get(tower)
                .ok()
                .map(|(kind, tier)| kind.0.stats((tier.0 + 1).min(TOWER_TIERS - 1)).range),
            _ => None,
        };
        if let Some(range) = range {
            strokes.circle(open.anchor, range, color.with_alpha(0.35));
        }
    }
}
