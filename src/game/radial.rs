//! The radial menu around a build spot or turret: entries, hit-testing, drawing.
//! Opens and closes on messages from `input`, which also decides what a click means.
//! Sole writer of `Menu`. Rules: docs/design/turrets.md#radial-menu

use bevy::prelude::*;

use crate::balance::{
    TowerKind, RADIAL_ENTRY_RADIUS, RADIAL_RADIUS, SELL_REFUND, TOWER_TIERS, Z_UI,
};
use crate::game::economy::Treasury;
use crate::game::input::{cursor_world, SellHold};
use crate::game::tower::{Investment, Targeting, Tier, TowerType};
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
}

pub struct Entry {
    pub action: Action,
    /// World offset from the anchor.
    pub offset: Vec2,
    pub enabled: bool,
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
        let open = self.open.as_ref()?;
        open.entries
            .iter()
            .find(|e| e.enabled && (open.anchor + e.offset).distance(world) <= RADIAL_ENTRY_RADIUS)
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

fn open(mut requests: MessageReader<OpenMenu>, mut menu: ResMut<Menu>, mut commands: Commands) {
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
    };
    let entries = actions
        .iter()
        .enumerate()
        .map(|(i, &action)| {
            let angle = std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 3.0;
            let offset = Vec2::from_angle(angle) * RADIAL_RADIUS;
            let label = commands
                .spawn((
                    Text2d::new(String::new()),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextLayout::new(Justify::Center, bevy::text::LineBreak::WordBoundary),
                    TextColor(Color::WHITE),
                    Transform::from_translation(offset.extend(1.0)),
                    ChildOf(root),
                ))
                .id();
            Entry {
                action,
                offset,
                enabled: false,
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

/// Labels and affordability follow the treasury and the turret every frame.
fn refresh(
    mut menu: ResMut<Menu>,
    treasury: Res<Treasury>,
    towers: Query<(&TowerType, &Tier, &Investment, &Targeting)>,
    mut labels: Query<(&mut Text2d, &mut TextColor)>,
) {
    let Some(open) = menu.open.as_mut() else {
        return;
    };
    let turret = match open.target {
        Target::Turret { tower, .. } => towers.get(tower).ok(),
        Target::Spot(_) => None,
    };

    for entry in &mut open.entries {
        let (text, enabled) = match (entry.action, turret) {
            (Action::Build(kind), _) => {
                let stats = kind.stats(0);
                (
                    format!("{}\n{} RM", stats.name, stats.cost),
                    treasury.marks >= stats.cost,
                )
            }
            (Action::Upgrade, Some((kind, tier, _, _))) => {
                let next = tier.0 + 1;
                if next < TOWER_TIERS {
                    let cost = kind.0.stats(next).cost;
                    (format!("Ausbau\n{cost} RM"), treasury.marks >= cost)
                } else {
                    ("Ausgebaut".to_string(), false)
                }
            }
            (Action::Sell, Some((_, _, investment, _))) => (
                format!(
                    "Verkaufen\n+{} RM",
                    (investment.0 as f32 * SELL_REFUND).round()
                ),
                true,
            ),
            (Action::Targeting, Some((_, _, _, targeting))) => {
                (format!("Ziel:\n{}", targeting.label()), true)
            }
            (_, None) => (String::new(), false),
        };
        entry.enabled = enabled;
        if let Ok((mut label, mut color)) = labels.get_mut(entry.label) {
            label.0 = text;
            color.0 = if enabled {
                Color::WHITE
            } else {
                Color::srgb(0.5, 0.5, 0.5)
            };
        }
    }
}

fn draw(
    mut gizmos: Gizmos,
    menu: Res<Menu>,
    hold: Res<SellHold>,
    towers: Query<(&TowerType, &Tier)>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let Some(open) = menu.open.as_ref() else {
        return;
    };
    let hovered = cursor_world(&camera, *window)
        .and_then(|c| menu.hit(c))
        .map(|e| e.action);

    for entry in &open.entries {
        let centre = open.anchor + entry.offset;
        let is_hovered = hovered == Some(entry.action);
        let color = match (entry.enabled, is_hovered) {
            (false, _) => Color::srgb(0.45, 0.45, 0.45),
            (true, true) => Color::srgb(1.0, 0.95, 0.65),
            (true, false) => Color::srgb(0.85, 0.85, 0.6),
        };
        gizmos.circle_2d(
            Isometry2d::from_translation(centre),
            RADIAL_ENTRY_RADIUS,
            color,
        );

        if entry.action == Action::Sell && hold.progress > 0.0 {
            gizmos.arc_2d(
                Isometry2d::from_translation(centre),
                std::f32::consts::TAU * hold.progress,
                RADIAL_ENTRY_RADIUS + 4.0,
                Color::srgb(0.9, 0.4, 0.3),
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
            gizmos.circle_2d(
                Isometry2d::from_translation(open.anchor),
                range,
                color.with_alpha(0.35),
            );
        }
    }
}
