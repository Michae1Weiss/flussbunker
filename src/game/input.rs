//! The only module that reads keyboard or mouse. Turns input into intent messages
//! in `Update`; the simulation consumes them in `FixedUpdate`.

use bevy::prelude::*;

use crate::balance::{TowerKind, TILE_SIZE};
use crate::game::map::RiverPath;
use crate::game::tower::Tower;
use crate::game_plugin::Frame;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SelectedTool>()
        .add_message::<BuildTower>()
        .add_message::<MoveUBoat>()
        .add_message::<ToggleTrim>()
        .add_message::<FireTorpedo>()
        .add_message::<CommissionUBoat>()
        .add_systems(
            Update,
            (select_tool, read_intents).chain().in_set(Frame::Input),
        )
        .add_systems(Update, draw_hover.in_set(Frame::Present));
}

/// Index into `TowerKind::ALL`; `kind` clamps it.
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
    /// World position.
    pub position: Vec2,
}

#[derive(Message)]
pub struct MoveUBoat {
    /// Arc length along `RiverPath`.
    pub along: f32,
}

#[derive(Message)]
pub struct ToggleTrim;

/// Sent every frame while the fire key is held; the boat rate-limits it.
#[derive(Message)]
pub struct FireTorpedo;

#[derive(Message)]
pub struct CommissionUBoat {
    pub free: bool,
}

fn select_tool(keys: Res<ButtonInput<KeyCode>>, mut tool: ResMut<SelectedTool>) {
    for (index, key) in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]
        .iter()
        .enumerate()
    {
        if keys.just_pressed(*key) {
            *tool = SelectedTool(index);
        }
    }
}

fn read_intents(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    tool: Res<SelectedTool>,
    path: Res<RiverPath>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut build: MessageWriter<BuildTower>,
    mut move_boat: MessageWriter<MoveUBoat>,
    mut trim: MessageWriter<ToggleTrim>,
    mut fire: MessageWriter<FireTorpedo>,
    mut commission: MessageWriter<CommissionUBoat>,
) {
    if keys.just_pressed(KeyCode::KeyT) || buttons.just_pressed(MouseButton::Right) {
        trim.write(ToggleTrim);
    }
    if keys.pressed(KeyCode::Space) {
        fire.write(FireTorpedo);
    }
    if keys.just_pressed(KeyCode::KeyN) {
        commission.write(CommissionUBoat { free: false });
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(world) = cursor_world(&camera, *window) else {
        return;
    };

    let (along, distance) = path.project(world);
    if path.contains(world) {
        move_boat.write(MoveUBoat { along });
    } else if distance > TILE_SIZE {
        build.write(BuildTower {
            kind: tool.kind(),
            position: world,
        });
    }
}

/// Cursor in world units; `None` when the cursor is outside the window.
pub fn cursor_world(camera: &(&Camera, &GlobalTransform), window: &Window) -> Option<Vec2> {
    let (camera, camera_transform) = *camera;
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

fn draw_hover(
    mut gizmos: Gizmos,
    tool: Res<SelectedTool>,
    path: Res<RiverPath>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    towers: Query<&Transform, With<Tower>>,
) {
    let Some(world) = cursor_world(&camera, *window) else {
        return;
    };

    if path.contains(world) {
        let (along, _) = path.project(world);
        let lane = Color::srgb(0.55, 0.90, 1.0);
        let here = path.point_at(along);
        gizmos.circle_2d(Isometry2d::from_translation(here), 6.0, lane);
        gizmos.line_2d(here, path.point_at(path.straight_end(along, false)), lane);
        return;
    }

    let occupied = towers
        .iter()
        .any(|t| t.translation.truncate().distance(world) < TILE_SIZE);
    let color = if occupied {
        Color::srgb(0.8, 0.3, 0.3)
    } else {
        Color::srgb(0.85, 0.85, 0.6)
    };
    gizmos.rect_2d(
        Isometry2d::from_translation(world),
        Vec2::splat(TILE_SIZE * 0.62),
        color,
    );
    if !occupied {
        gizmos.circle_2d(
            Isometry2d::from_translation(world),
            tool.kind().stats().range,
            color.with_alpha(0.35),
        );
    }
}
