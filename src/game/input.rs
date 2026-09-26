//! The only module that reads keyboard or mouse. Turns input into intent messages
//! in `Update`; the simulation consumes them in `FixedUpdate`. Priority: radial menu
//! before world. Sole writer of `SellHold`, `Hovered`, `Pointing`, `FireArmed`.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::balance::{TowerKind, BUILD_SPOT_RADIUS, SELL_HOLD_SECONDS, UBOAT_LENGTH};
use crate::game::map::{BuildSpot, RiverPath};
use crate::game::notes::ShowNote;
use crate::game::palette::{INK, RED};
use crate::game::radial::{Action, CloseMenu, Menu, OpenMenu, Target};
use crate::game::stroke::Strokes;
use crate::game::tower::Occupied;
use crate::game::uboat::{Station, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<SellHold>()
        .init_resource::<FireArmed>()
        .add_systems(OnEnter(Screen::Gameplay), disarm_fire)
        .add_message::<BuildTower>()
        .add_message::<UpgradeTower>()
        .add_message::<SellTower>()
        .add_message::<CycleTargeting>()
        .add_message::<MoveUBoat>()
        .add_message::<ToggleTrim>()
        .add_message::<FireTorpedo>()
        .add_message::<UpgradeBoat>()
        .add_message::<ToggleSpeed>()
        .add_message::<StartWave>()
        .add_message::<TogglePause>()
        .add_message::<ToggleOverlay>()
        .add_message::<ToggleMute>()
        .add_message::<Refused>()
        .init_resource::<Hovered>()
        .init_resource::<Pointing>()
        .add_systems(Update, (read_intents, track_hover).in_set(Frame::Input))
        .add_systems(Update, draw_hover.in_set(Frame::Present));
}

/// The enabled radial-menu entry under the cursor, if any.
#[derive(Resource, Default)]
pub struct Hovered(pub Option<Action>);

/// What a click would act on, for the cursor's look and the hover sound.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointing {
    #[default]
    Nothing,
    Water,
    Spot,
    Turret,
    Boat,
    /// An enabled radial-menu entry.
    Entry,
}

/// What lies at a point of the map, as a click there would take it.
enum Under {
    DockedBoat(Entity, Vec2),
    Spot {
        spot: Entity,
        tower: Option<Entity>,
        at: Vec2,
    },
    Water(f32),
}

impl Under {
    fn pointing(&self) -> Pointing {
        match self {
            Under::DockedBoat(..) => Pointing::Boat,
            Under::Spot { tower: None, .. } => Pointing::Spot,
            Under::Spot { .. } => Pointing::Turret,
            Under::Water(_) => Pointing::Water,
        }
    }
}

#[derive(SystemParam)]
struct Ground<'w, 's> {
    path: Res<'w, RiverPath>,
    wave: Res<'w, WaveState>,
    spots: Query<'w, 's, (Entity, &'static Transform, Option<&'static Occupied>), With<BuildSpot>>,
    boats: Query<'w, 's, (Entity, &'static Transform, &'static Station), With<UBoat>>,
}

impl Ground<'_, '_> {
    /// The docked boat before a spot before the river; spots only after the prologue.
    fn under(&self, world: Vec2) -> Option<Under> {
        let docked = self.boats.iter().find(|(_, t, station)| {
            station.is_docked(&self.path)
                && t.translation.truncate().distance(world) <= UBOAT_LENGTH
        });
        if let Some((boat, transform, _)) = docked {
            return Some(Under::DockedBoat(boat, transform.translation.truncate()));
        }
        if let Some((spot, transform, occupied)) =
            spot_at(world, self.spots.iter()).filter(|_| !self.wave.prologue)
        {
            return Some(Under::Spot {
                spot,
                tower: occupied.map(|o| o.0),
                at: transform.translation.truncate(),
            });
        }
        self.path
            .contains(world)
            .then(|| Under::Water(self.path.project(world).0))
    }
}

/// Holding Space fires only after a press seen in gameplay; the press that started the run
/// is still held on its first frames.
#[derive(Resource, Default)]
struct FireArmed(bool);

fn disarm_fire(mut armed: ResMut<FireArmed>) {
    armed.0 = false;
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

/// Enter, or the next-wave button: the build phase ends.
#[derive(Message)]
pub struct StartWave;

/// Esc with no radial menu open: the pause menu.
#[derive(Message)]
pub struct TogglePause;

/// Tab: the tactical overlay.
#[derive(Message)]
pub struct ToggleOverlay;

/// M: sound off and on.
#[derive(Message)]
pub struct ToggleMute;

/// A click on a menu entry the treasury cannot pay for.
#[derive(Message)]
pub struct Refused {
    pub at: Vec2,
}

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
    start: MessageWriter<'w, StartWave>,
    pause: MessageWriter<'w, TogglePause>,
    overlay: MessageWriter<'w, ToggleOverlay>,
    mute: MessageWriter<'w, ToggleMute>,
    refused: MessageWriter<'w, Refused>,
    note: MessageWriter<'w, ShowNote>,
}

fn read_intents(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    menu: Res<Menu>,
    ground: Ground,
    mut hold: ResMut<SellHold>,
    mut armed: ResMut<FireArmed>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    paper_buttons: Query<&Interaction, With<Button>>,
    mut intents: Intents,
) {
    if keys.just_pressed(KeyCode::KeyT) || buttons.just_pressed(MouseButton::Right) {
        intents.trim.write(ToggleTrim);
    }
    armed.0 |= keys.just_pressed(KeyCode::Space);
    if armed.0 && keys.pressed(KeyCode::Space) {
        intents.fire.write(FireTorpedo);
    }
    if keys.just_pressed(KeyCode::Escape) {
        if menu.open.is_some() {
            intents.close_menu.write(CloseMenu);
        } else {
            intents.pause.write(TogglePause);
        }
    }
    if keys.just_pressed(KeyCode::KeyF) {
        intents.speed.write(ToggleSpeed);
    }
    if keys.just_pressed(KeyCode::Enter) {
        intents.start.write(StartWave);
    }
    if keys.just_pressed(KeyCode::Tab) {
        intents.overlay.write(ToggleOverlay);
    }
    if keys.just_pressed(KeyCode::KeyM) {
        intents.mute.write(ToggleMute);
    }

    // A click on a paper button belongs to the button.
    if paper_buttons.iter().any(|i| *i != Interaction::None) {
        return;
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
                if let Some(disabled) = cursor.and_then(|c| menu.hit_any(c)) {
                    let at = open.anchor + disabled.offset;
                    intents.note.write(ShowNote {
                        text: "Zu teuer",
                        at,
                    });
                    intents.refused.write(Refused { at });
                } else {
                    intents.close_menu.write(CloseMenu);
                }
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

    match ground.under(world) {
        Some(Under::DockedBoat(boat, at)) => {
            intents.open_menu.write(OpenMenu {
                target: Target::Boat(boat),
                anchor: at,
            });
        }
        Some(Under::Spot { spot, tower, at }) => {
            let target = match tower {
                Some(tower) => Target::Turret { spot, tower },
                None => Target::Spot(spot),
            };
            intents.open_menu.write(OpenMenu { target, anchor: at });
        }
        Some(Under::Water(along)) => {
            intents.move_boat.write(MoveUBoat { along });
        }
        None => {}
    }
}

fn track_hover(
    menu: Res<Menu>,
    ground: Ground,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    paper_buttons: Query<&Interaction, With<Button>>,
    mut hovered: ResMut<Hovered>,
    mut pointing: ResMut<Pointing>,
) {
    let cursor = cursor_world(&camera, *window)
        .filter(|_| paper_buttons.iter().all(|i| *i == Interaction::None));
    let entry = cursor.and_then(|c| menu.hit(c)).map(|e| e.action);
    if hovered.0 != entry {
        hovered.0 = entry;
    }
    let now = match (cursor, menu.open.is_some()) {
        (Some(_), true) if entry.is_some() => Pointing::Entry,
        (Some(world), false) => ground
            .under(world)
            .map_or(Pointing::Nothing, |u| u.pointing()),
        _ => Pointing::Nothing,
    };
    if *pointing != now {
        *pointing = now;
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
    mut strokes: ResMut<Strokes>,
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
        strokes.circle(transform.translation.truncate(), BUILD_SPOT_RADIUS, INK);
        return;
    }

    if path.contains(world) {
        let (along, _) = path.project(world);
        let lane = RED;
        let here = path.point_at(along);
        strokes.circle(here, 6.0, lane);
        strokes.line(here, path.point_at(path.straight_end(along, false)), lane);
    }
}
