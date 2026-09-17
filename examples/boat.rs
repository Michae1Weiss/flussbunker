//! Feel harness for M1: the boat alone on the river with a stream of dummy ships.
//! `cargo run --example boat`. Left-click moves, right-click dives, Space fires.

use bevy::prelude::*;
use flussbunker::balance::EnemyKind;
use flussbunker::game::enemy::{depth_charge_bundle, enemy_bundle};
use flussbunker::game::map::RiverPath;
use flussbunker::game::{combat, economy, enemy, input, map, projectile, uboat};
use flussbunker::harness;

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
            input::plugin,
        ))
        .insert_resource(Stream {
            light: Timer::from_seconds(4.0, TimerMode::Repeating),
            heavy: Timer::from_seconds(24.0, TimerMode::Repeating),
        })
        .add_systems(Update, stream_ships)
        .run()
}

#[derive(Resource)]
struct Stream {
    light: Timer,
    heavy: Timer,
}

fn stream_ships(
    time: Res<Time>,
    mut commands: Commands,
    mut stream: ResMut<Stream>,
    path: Res<RiverPath>,
) {
    if stream.light.tick(time.delta()).just_finished() {
        commands.spawn(enemy_bundle(
            EnemyKind::MotorTorpedoBoat,
            path.spawn_point(),
        ));
    }
    if stream.heavy.tick(time.delta()).just_finished() {
        let mut ship = commands.spawn(enemy_bundle(EnemyKind::Destroyer, path.spawn_point()));
        if let Some(charges) = depth_charge_bundle(EnemyKind::Destroyer) {
            ship.insert(charges);
        }
    }
}
