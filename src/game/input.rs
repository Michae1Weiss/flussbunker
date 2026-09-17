//! The only module that reads keyboard or mouse. Turns input into intent messages
//! in `Update`; the simulation consumes them in `FixedUpdate`. Priority: radial menu
//! before world. Sole writer of `SellHold`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::balance::{TowerKind, BUILD_SPOT_RADIUS, SELL_HOLD_SECONDS, UBOAT_LENGTH};
use crate::game::map::{BuildSpot, RiverPath};
use crate::game::radial::{Action, CloseMenu, Menu, OpenMenu, Target};
use crate::game::tower::Occupied;
use crate::game::uboat::{Station, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;

pub fn plugin(app: &mut App) {
    app.init_resource::<SellHold>()
        .add_message::<BuildTower>()
        .add_message::<UpgradeTower>()
        .add_message::<SellTower>()
        .add_message::<CycleTargeting>()
        .add_message::<MoveUBoat>()
        .add_message::<ToggleTrim>()
        .add_message::<FireTorpedo>()
        .add_message::<UpgradeBoat>()
        .add_message::<ToggleSpeed>()
        .add_systems(Update, read_intents.in_set(Frame::Input))
        .add_systems(Update, draw_hover.in_set(Frame::Present));
}

/// Progress of a hold-to-sell, 0 to 1.
#[derive(Resource, Default)]
pub struct SellHold {
    pub progress: f32,
}

#[derive(Message)]
pub struct BuildTower {
    pub kind: TowerKind,
    /// A `BuildSpot` entity.
    pub spot: Entity,
}

#[derive(Message)]
pub struct UpgradeTower {
    pub tower: Entity,
}

#[derive(Message)]
pub struct SellTower {
    pub tower: Entity,
}

#[derive(Message)]
pub struct CycleTargeting {
    pub tower: Entity,
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

/// Buy the next boat model; the boat must be docked.
#[derive(Message)]
pub struct UpgradeBoat;

/// 1x / 2x game pace.
#[derive(Message)]
pub struct ToggleSpeed;

#[derive(SystemParam)]
struct Intents<'w> {
    build: MessageWriter<'w, BuildTower>,
    upgrade: MessageWriter<'w, UpgradeTower>,
    sell: MessageWriter<'w, SellTower>,
    cycle: MessageWriter<'w, CycleTargeting>,
    open_menu: MessageWriter<'w, OpenMenu>,
    close_menu: MessageWriter<'w, CloseMenu>,
    move_boat: MessageWriter<'w, MoveUBoat>,
    trim: MessageWriter<'w, ToggleTrim>,
    fire: MessageWriter<'w, FireTorpedo>,
    upgrade_boat: MessageWriter<'w, UpgradeBoat>,
    speed: MessageWriter<'w, ToggleSpeed>,
}

fn read_intents(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    path: Res<RiverPath>,
    menu: Res<Menu>,
    wave: Res<WaveState>,
    mut hold: ResMut<SellHold>,
    spots: Query<(Entity, &Transform, Option<&Occupied>), With<BuildSpot>>,
    boats: Query<(Entity, &Transform, &Station), With<UBoat>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut intents: Intents,
) {
    if keys.just_pressed(KeyCode::KeyT) || buttons.just_pressed(MouseButton::Right) {
        intents.trim.write(ToggleTrim);
    }
    if keys.pressed(KeyCode::Space) {
        intents.fire.write(FireTorpedo);
    }
    if keys.just_pressed(KeyCode::Escape) {
        intents.close_menu.write(CloseMenu);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        intents.speed.write(ToggleSpeed);
    }

    let cursor = cursor_world(&camera, *window);

    if let Some(open) = menu.open.as_ref() {
        let hit = cursor.and_then(|c| menu.hit(c)).map(|e| e.action);
        let tower = match open.target {
            Target::Turret { tower, .. } => Some(tower),
            Target::Spot(_) | Target::Boat(_) => None,
        };

        // Holding on the sell entry fills the ring; leaving it or releasing cancels.
        if buttons.pressed(MouseButton::Left) && hit == Some(Action::Sell) {
            hold.progress += time.delta_secs() / SELL_HOLD_SECONDS;
            if hold.progress >= 1.0 {
                hold.progress = 0.0;
                if let Some(tower) = tower {
                    intents.sell.write(SellTower { tower });
                }
                intents.close_menu.write(CloseMenu);
            }
            return;
        }
        hold.progress = 0.0;

        if !buttons.just_pressed(MouseButton::Left) {
            return;
        }
        match (hit, open.target, tower) {
            (Some(Action::Build(kind)), Target::Spot(spot), _) => {
                intents.build.write(BuildTower { kind, spot });
                intents.close_menu.write(CloseMenu);
            }
            (Some(Action::Upgrade), _, Some(tower)) => {
                intents.upgrade.write(UpgradeTower { tower });
                intents.close_menu.write(CloseMenu);
            }
            (Some(Action::UpgradeBoat), _, _) => {
                intents.upgrade_boat.write(UpgradeBoat);
                intents.close_menu.write(CloseMenu);
            }
            (Some(Action::Targeting), _, Some(tower)) => {
                intents.cycle.write(CycleTargeting { tower });
            }
            (Some(Action::Sell), _, _) => {}
            _ => {
                intents.close_menu.write(CloseMenu);
            }
        }
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(world) = cursor else {
        return;
    };

    let docked_boat = boats.iter().find(|(_, t, station)| {
        station.is_docked(&path) && t.translation.truncate().distance(world) <= UBOAT_LENGTH
    });
    if let Some((boat, transform, _)) = docked_boat {
        intents.open_menu.write(OpenMenu {
            target: Target::Boat(boat),
            anchor: transform.translation.truncate(),
        });
    } else if let Some((spot, transform, occupied)) =
        spot_at(world, spots.iter()).filter(|_| !wave.prologue)
    {
        let target = match occupied {
            Some(occupied) => Target::Turret {
                spot,
                tower: occupied.0,
            },
            None => Target::Spot(spot),
        };
        intents.open_menu.write(OpenMenu {
            target,
            anchor: transform.translation.truncate(),
        });
    } else if path.contains(world) {
        let (along, _) = path.project(world);
        intents.move_boat.write(MoveUBoat { along });
    }
}

/// Cursor in world units; `None` when the cursor is outside the window.
pub fn cursor_world(camera: &(&Camera, &GlobalTransform), window: &Window) -> Option<Vec2> {
    let (camera, camera_transform) = *camera;
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

/// The build spot under `world`, if any.
fn spot_at<'a, O>(
    world: Vec2,
    mut spots: impl Iterator<Item = (Entity, &'a Transform, O)>,
) -> Option<(Entity, &'a Transform, O)> {
    spots.find(|(_, t, _)| t.translation.truncate().distance(world) <= BUILD_SPOT_RADIUS)
}

fn draw_hover(
    mut gizmos: Gizmos,
    path: Res<RiverPath>,
    menu: Res<Menu>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    spots: Query<(Entity, &Transform, Has<Occupied>), With<BuildSpot>>,
) {
    if menu.open.is_some() {
        return;
    }
    let Some(world) = cursor_world(&camera, *window) else {
        return;
    };

    if let Some((_, transform, _)) = spot_at(world, spots.iter()) {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            BUILD_SPOT_RADIUS,
            Color::srgb(0.85, 0.85, 0.6),
        );
        return;
    }

    if path.contains(world) {
        let (along, _) = path.project(world);
        let lane = Color::srgb(0.55, 0.90, 1.0);
        let here = path.point_at(along);
        gizmos.circle_2d(Isometry2d::from_translation(here), 6.0, lane);
        gizmos.line_2d(here, path.point_at(path.straight_end(along, false)), lane);
    }
}
