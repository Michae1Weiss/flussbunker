//! Player intent. The only module that reads the keyboard or the mouse.
//!
//! It does not build anything. It turns clicks into three messages —
//! `BuildTower`, `OrderUBoat`, `LaunchUBoat` — and the simulation decides
//! whether they are legal. That split earns its keep three times over:
//!
//! 1. **The `FixedUpdate` input trap.** `just_pressed` flips once per *frame*,
//!    while `FixedUpdate` may run twice or not at all — reading input there
//!    drops or duplicates clicks. Sampling in `Update` and buffering the intent
//!    as a message is the standard fix, and it is invisible here because the
//!    message *is* the buffer.
//! 2. **Affordability, legality and cost live with the thing being built**, not
//!    with the mouse. `tower.rs` owns what a tower costs.
//! 3. **Replays, AI and tests** can write the same messages. A test that builds
//!    a tower never needs a fake cursor.

use bevy::prelude::*;

use crate::balance::{TowerKind, TILE_SIZE};
use crate::game::map::{MapGrid, RiverPath, Tile};
use crate::game::tower::Tower;
use crate::game_plugin::Frame;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SelectedTool>()
        .add_message::<BuildTower>()
        .add_message::<OrderUBoat>()
        .add_message::<LaunchUBoat>()
        .add_systems(
            Update,
            (select_tool, issue_orders).chain().in_set(Frame::Input),
        )
        .add_systems(Update, draw_hover.in_set(Frame::Present));
}

/// What a left click currently means.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTool {
    #[default]
    Flak,
    Coastal,
}

impl SelectedTool {
    pub fn tower_kind(self) -> TowerKind {
        match self {
            SelectedTool::Flak => TowerKind::Flak,
            SelectedTool::Coastal => TowerKind::Coastal,
        }
    }
}

#[derive(Message)]
pub struct BuildTower {
    pub kind: TowerKind,
    pub cell: IVec2,
}

#[derive(Message)]
pub struct OrderUBoat;

#[derive(Message)]
pub struct LaunchUBoat {
    /// Index into `RiverPath` — the station to hold.
    pub station: usize,
}

fn select_tool(keys: Res<ButtonInput<KeyCode>>, mut tool: ResMut<SelectedTool>) {
    if keys.just_pressed(KeyCode::Digit1) {
        *tool = SelectedTool::Flak;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *tool = SelectedTool::Coastal;
    }
}

fn issue_orders(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    tool: Res<SelectedTool>,
    grid: Res<MapGrid>,
    path: Res<RiverPath>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut build: MessageWriter<BuildTower>,
    mut order: MessageWriter<OrderUBoat>,
    mut launch: MessageWriter<LaunchUBoat>,
) {
    if keys.just_pressed(KeyCode::KeyU) {
        order.write(OrderUBoat);
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cell) = cursor_cell(&camera, *window, &grid) else {
        return;
    };

    // The click means different things over land and over water. One button,
    // context-sensitive — fewer keys to teach than a separate "sortie" mode.
    match grid.tile(cell) {
        Some(Tile::Land) => {
            build.write(BuildTower {
                kind: tool.tower_kind(),
                cell,
            });
        }
        Some(Tile::River) => {
            if let Some(station) = path.index_of(cell) {
                launch.write(LaunchUBoat { station });
            }
        }
        _ => {}
    }
}

/// Cursor position as a grid cell, if the cursor is over the window at all.
///
/// A free function returning `Option`, not a system: systems cannot return
/// `Option`, and two callers need this. `viewport_to_world_2d` returns a
/// `Result` in 0.19 — `.ok()?` folds it into the same "no cursor, no problem"
/// path rather than making callers handle an error that is not one.
pub fn cursor_cell(
    camera: &(&Camera, &GlobalTransform),
    window: &Window,
    grid: &MapGrid,
) -> Option<IVec2> {
    let (camera, camera_transform) = *camera;
    let cursor = window.cursor_position()?;
    let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    Some(grid.world_to_grid(world))
}

/// Hover feedback: the build square and the range it would cover.
///
/// Item 8 on the juice checklist and the single highest-value one for a tower
/// defense: a player who cannot see a battery's range cannot plan, and a player
/// who cannot plan is watching a screensaver. Gizmos are immediate-mode, so
/// this needs no entities and no cleanup.
fn draw_hover(
    mut gizmos: Gizmos,
    tool: Res<SelectedTool>,
    grid: Res<MapGrid>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    towers: Query<&Transform, With<Tower>>,
) {
    let Some(cell) = cursor_cell(&camera, *window, &grid) else {
        return;
    };
    let Some(tile) = grid.tile(cell) else { return };
    let centre = grid.grid_to_world(cell);

    match tile {
        Tile::Land => {
            let occupied = towers
                .iter()
                .any(|t| grid.world_to_grid(t.translation.truncate()) == cell);
            let color = if occupied {
                Color::srgb(0.8, 0.3, 0.3)
            } else {
                Color::srgb(0.85, 0.85, 0.6)
            };
            gizmos.rect_2d(Isometry2d::from_translation(centre), Vec2::splat(TILE_SIZE), color);
            if !occupied {
                gizmos.circle_2d(
                    Isometry2d::from_translation(centre),
                    tool.tower_kind().stats().range,
                    color.with_alpha(0.35),
                );
            }
        }
        Tile::River => {
            gizmos.circle_2d(
                Isometry2d::from_translation(centre),
                TILE_SIZE * 0.4,
                Color::srgb(0.5, 0.85, 1.0),
            );
        }
        Tile::Bunker => {}
    }
}
