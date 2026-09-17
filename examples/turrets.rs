//! Feel harness for M2: the radial menu, building, upgrading, selling and targeting,
//! with a fat treasury and a stream of dummy ships. `cargo run --example turrets`.

use bevy::prelude::*;
use flussbunker::balance::EnemyKind;
use flussbunker::game::economy::Treasury;
use flussbunker::game::enemy::enemy_bundle;
use flussbunker::game::map::RiverPath;
use flussbunker::game::{combat, economy, enemy, input, map, projectile, radial, tower, uboat};
use flussbunker::harness;
use flussbunker::screens::Screen;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(harness)
        .add_plugins((
            map::plugin,
            combat::plugin,
            enemy::plugin,
            projectile::plugin,
            uboat::plugin,
            economy::plugin,
            tower::plugin,
            radial::plugin,
            input::plugin,
        ))
        .insert_resource(Stream(Timer::from_seconds(3.0, TimerMode::Repeating)))
        .add_systems(OnEnter(Screen::Gameplay), fund)
        .add_systems(Update, stream_ships)
        .run()
}

#[derive(Resource)]
struct Stream(Timer);

fn fund(mut treasury: ResMut<Treasury>) {
    treasury.marks = 3000;
}

fn stream_ships(
    time: Res<Time>,
    mut commands: Commands,
    mut stream: ResMut<Stream>,
    path: Res<RiverPath>,
) {
    if !stream.0.tick(time.delta()).just_finished() {
        return;
    }
    let kind = match (time.elapsed_secs() as usize / 3) % 3 {
        0 => EnemyKind::MotorTorpedoBoat,
        1 => EnemyKind::Corvette,
        _ => EnemyKind::Destroyer,
    };
    commands.spawn(enemy_bundle(kind, path.spawn_point()));
}
