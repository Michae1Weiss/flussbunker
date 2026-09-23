//! The M8 style test as a program: every ship, boat and emplacement tier on the river at
//! game scale, notes filled, the radial menu open; time runs briefly so barrels turn, then
//! freezes on a volley of muzzle flashes and a torpedo bursting on the bank. Saves
//! `target/lineup/<w>x<h>.png` plus `-overlay.png` and quits.
//! `cargo run --example lineup -- 1280 800`, or `-- full` for the screen itself.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{MonitorSelection, WindowMode, WindowResolution};
use flussbunker::balance::{
    EnemyKind, TowerKind, TOWER_SIZE, UBOAT_BEAM, UBOAT_LENGTH, UBOAT_LENGTH_STEP,
};
use flussbunker::game::art::Art;
use flussbunker::game::combat::{Fired, Weapon};
use flussbunker::game::enemy::{enemy_bundle, Along};
use flussbunker::game::input::ToggleOverlay;
use flussbunker::game::map::{tile_to_world, BuildSpot, RiverPath};
use flussbunker::game::projectile::TorpedoLost;
use flussbunker::game::radial::{OpenMenu, Target};
use flussbunker::game::tower::{tower_bundle, Aim, Constructing, Tier, Tower, TowerType};
use flussbunker::{game, harness};

const SHIPS: [EnemyKind; 6] = [
    EnemyKind::MotorTorpedoBoat,
    EnemyKind::Corvette,
    EnemyKind::LandingCraft,
    EnemyKind::Destroyer,
    EnemyKind::Monitor,
    EnemyKind::GoldTransport,
];

fn main() -> AppExit {
    let full = std::env::args().any(|a| a == "full");
    let args: Vec<u32> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let (width, height) = match args[..] {
        [w, h] => (w, h),
        _ => (1280, 800),
    };
    let name = if full {
        "full".to_string()
    } else {
        format!("{width}x{height}")
    };
    std::fs::create_dir_all("target/lineup").expect("target/ is writable");
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "lineup".to_string(),
                        resolution: WindowResolution::new(width, height),
                        resizable: false,
                        mode: if full {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                        } else {
                            WindowMode::Windowed
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_linear()),
        )
        .add_plugins((harness, game::plugin))
        .insert_resource(Shot(format!("target/lineup/{name}")))
        .add_systems(Update, script)
        .run()
}

#[derive(Resource)]
struct Shot(String);

#[allow(clippy::too_many_arguments)]
fn script(
    mut frame: Local<u32>,
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    path: Res<RiverPath>,
    art: Res<Art>,
    shot: Res<Shot>,
    spots: Query<(Entity, &Transform), With<BuildSpot>>,
    mut towers: Query<(Entity, &Transform, &TowerType, &Tier, &Aim, &mut Sprite), With<Tower>>,
    ships: Query<&Transform, Without<Tower>>,
    mut fired: MessageWriter<Fired>,
    mut lost: MessageWriter<TorpedoLost>,
    mut menus: MessageWriter<OpenMenu>,
    mut overlay: MessageWriter<ToggleOverlay>,
    mut exit: MessageWriter<AppExit>,
) {
    *frame += 1;
    match *frame {
        5 => {
            time.pause();
            let along = |i: usize| path.length() * (0.08 + 0.075 * i as f32);
            for (i, &kind) in SHIPS.iter().enumerate() {
                let s = along(i);
                let heading = path.tangent_at(s);
                commands
                    .spawn(enemy_bundle(kind, path.point_at(s)))
                    .insert((
                        Along(s),
                        Transform::from_translation(path.point_at(s).extend(5.0))
                            .with_rotation(Quat::from_rotation_arc_2d(Vec2::Y, heading)),
                    ));
            }
            for (model, key) in [
                (1.0, "sprites/boats/viic.png"),
                (2.0, "sprites/boats/ixc.png"),
            ] {
                let s = along(6 + model as usize);
                commands.spawn((
                    Sprite {
                        image: art.image(key).unwrap_or_default(),
                        custom_size: Some(Vec2::new(
                            UBOAT_BEAM,
                            UBOAT_LENGTH + UBOAT_LENGTH_STEP * model,
                        )),
                        ..default()
                    },
                    Transform::from_translation(path.point_at(s).extend(5.0))
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::Y, path.tangent_at(s))),
                ));
            }
            let mut spots: Vec<_> = spots.iter().collect();
            spots.sort_by(|a, b| a.1.translation.x.total_cmp(&b.1.translation.x));
            for (i, (_, spot)) in spots.iter().skip(1).take(7).enumerate() {
                let kind = TowerKind::ALL[i % 3];
                commands
                    .spawn(tower_bundle(kind, spot.translation.truncate()))
                    .insert(Tier((i / 3).min(2) as u8))
                    .remove::<Constructing>();
            }
            if let Some((spot, transform)) = spots.first() {
                menus.write(OpenMenu {
                    target: Target::Spot(*spot),
                    anchor: transform.translation.truncate(),
                });
            }
        }
        10 => {
            for (_, _, _, tier, _, mut sprite) in &mut towers {
                sprite.color.set_alpha(1.0);
                sprite.custom_size = Some(Vec2::splat(TOWER_SIZE[tier.0 as usize]));
            }
        }
        60 => time.unpause(),
        88 => {
            time.pause();
            for (tower, transform, kind, tier, aim, _) in &towers {
                let origin = transform.translation.truncate();
                let dir = aim.0.and_then(|e| ships.get(e).ok()).map_or(Vec2::Y, |t| {
                    (t.translation.truncate() - origin).normalize_or(Vec2::Y)
                });
                fired.write(Fired {
                    weapon: Weapon::Turret(kind.0),
                    shooter: tower,
                    from: origin + dir * TOWER_SIZE[tier.0 as usize] * 0.5,
                    dir,
                });
            }
            lost.write(TorpedoLost {
                at: tile_to_world(5.0, 4.4),
            });
        }
        90 => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("{}.png", shot.0)));
        }
        100 => {
            overlay.write(ToggleOverlay);
        }
        140 => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("{}-overlay.png", shot.0)));
        }
        180 => {
            info!("lineup saved to {}.png and -overlay.png", shot.0);
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
