//! Player intent. The only module that reads the keyboard or the mouse.
//!
//! It does not build anything and does not move anything. It turns input into
//! four messages and lets the simulation decide what is legal:
//! `BuildTower`, `MoveUBoat`, `ToggleTrim`, `CommissionUBoat`.
//!
//! Why the split earns its keep:
//!
//! 1. **The `FixedUpdate` input trap.** `just_pressed` flips once per *frame*,
//!    while `FixedUpdate` may run twice or not at all — reading input there
//!    drops or duplicates clicks. Sampling in `Update` and buffering the intent
//!    as a message is the standard fix, and it is invisible here because the
//!    message *is* the buffer.
//! 2. Cost and legality live with the thing being built, not with the mouse.
//! 3. Replays, AI and tests write the same messages, with no fake cursor.
//!
//! v2 changed one binding, and it was the most important change in the game:
//! **clicking water now moves U-96 there.** The playtest expected clicking a
//! river tile to *do something at that tile*. It previously performed an
//! invisible purchase. Same click, right verb.

use bevy::prelude::*;

use crate::balance::{TowerKind, TILE_SIZE};
use crate::game::map::{MapGrid, RiverPath, Tile};
use crate::game::tower::Tower;
use crate::game_plugin::Frame;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SelectedTool>()
        .add_message::<BuildTower>()
        .add_message::<MoveUBoat>()
        .add_message::<ToggleTrim>()
        .add_message::<CommissionUBoat>()
        .add_systems(Update, (select_tool, issue_orders).chain().in_set(Frame::Input))
        .add_systems(Update, draw_hover.in_set(Frame::Present));
}

/// What a left click on land currently means.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedTool(pub usize);

impl SelectedTool {
    pub fn kind(self) -> TowerKind {
        TowerKind::ALL[self.0.min(TowerKind::ALL.len() - 1)]
    }
}

#[derive(Message)]
pub struct BuildTower {
    pub kind: TowerKind,
    pub cell: IVec2,
}

#[derive(Message)]
pub struct MoveUBoat {
    /// Index into `RiverPath`.
    pub station: usize,
}

#[derive(Message)]
pub struct ToggleTrim;

#[derive(Message)]
pub struct CommissionUBoat {
    pub free: bool,
}

fn select_tool(keys: Res<ButtonInput<KeyCode>>, mut tool: ResMut<SelectedTool>) {
    for (index, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3].iter().enumerate() {
        if keys.just_pressed(*key) {
            *tool = SelectedTool(index);
        }
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
    mut move_boat: MessageWriter<MoveUBoat>,
    mut trim: MessageWriter<ToggleTrim>,
    mut commission: MessageWriter<CommissionUBoat>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        trim.write(ToggleTrim);
    }
    if keys.just_pressed(KeyCode::KeyN) {
        commission.write(CommissionUBoat { free: false });
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cell) = cursor_cell(&camera, *window, &grid) else {
        return;
    };

    // One button, context-sensitive: land builds, water commands the boat.
    // Fewer keys to teach, and it matches where the hand already wants to click.
    match grid.tile(cell) {
        Some(Tile::Land) => {
            build.write(BuildTower { kind: tool.kind(), cell });
        }
        Some(Tile::River) | Some(Tile::Bunker) => {
            if let Some(station) = path.index_of(cell) {
                move_boat.write(MoveUBoat { station });
            }
        }
        None => {}
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

/// Hover feedback. The highest-value juice in a tower defense: a player who
/// cannot see a battery's range cannot plan, and a player who cannot plan is
/// watching a screensaver.
///
/// v2 adds the firing-lane preview over water — hovering a river tile draws the
/// straight a torpedo would run from there. That is what teaches the whole
/// positioning mechanic without a single line of tutorial text.
fn draw_hover(
    mut gizmos: Gizmos,
    tool: Res<SelectedTool>,
    grid: Res<MapGrid>,
    path: Res<RiverPath>,
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
                    tool.kind().stats().range,
                    color.with_alpha(0.35),
                );
            }
        }
        Tile::River | Tile::Bunker => {
            let lane = Color::srgb(0.55, 0.90, 1.0);
            gizmos.rect_2d(Isometry2d::from_translation(centre), Vec2::splat(TILE_SIZE), lane);
            if let Some(end) = firing_lane_end(&path, &grid, cell) {
                gizmos.line_2d(centre, end, lane);
            }
        }
    }
}

/// Where a torpedo fired from `cell` would end up: follow the river upstream
/// while the direction does not change, i.e. to the end of this straight.
///
/// Pure function of the map, no ECS. That makes it trivially testable and
/// means the preview and the real torpedo cannot disagree about geometry —
/// they are reading the same river.
pub fn firing_lane_end(path: &RiverPath, grid: &MapGrid, cell: IVec2) -> Option<Vec2> {
    let index = path.index_of(cell)?;
    let upstream = index.checked_sub(1)?;
    let heading = path.cells[upstream] - path.cells[index];

    let mut last = upstream;
    let mut probe = upstream;
    while let Some(next) = probe.checked_sub(1) {
        if path.cells[next] - path.cells[probe] != heading {
            break;
        }
        last = next;
        probe = next;
    }
    Some(grid.grid_to_world(path.cells[last]))
}
