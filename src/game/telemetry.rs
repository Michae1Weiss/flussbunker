//! One CSV row per run, written locally when gameplay ends. Reads messages and state,
//! writes nothing the game uses, never uploads. Sole writer of `RunLog`.
//! Fields: docs/PLAYTEST.md#telemetry

use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::balance::{TowerKind, WAVES};
use crate::game::bunker::Integrity;
use crate::game::combat::{Damage, LastHitBy, Sunk, Weapon};
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::input::{SellTower, UpgradeTower};
use crate::game::projectile::{Torpedo, TorpedoLost};
use crate::game::tower::{Tower, TowerType};
use crate::game::uboat::{Trim, UBoat};
use crate::game::wave::WaveState;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<RunLog>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(Update, record.run_if(in_state(Screen::Gameplay)))
        .add_systems(OnExit(Screen::Gameplay), write_row)
        .add_observer(on_sunk);
}

#[derive(Resource, Default, Debug)]
pub struct RunLog {
    pub seconds: f32,
    pub torpedoes_fired: u32,
    pub torpedoes_hit: u32,
    pub torpedoes_lost: u32,
    pub first_torpedo_at: Option<f32>,
    pub first_build_at: Option<f32>,
    pub first_dive_at: Option<f32>,
    pub turrets_built: [u32; 3],
    pub upgrades: u32,
    pub sells: u32,
    pub boat_sunk: u32,
    pub ram_kills: u32,
    pub chain_kills: u32,
    pub kills: u32,
}

fn reset(mut log: ResMut<RunLog>) {
    *log = RunLog::default();
}

fn record(
    time: Res<Time>,
    mut log: ResMut<RunLog>,
    new_torpedoes: Query<(), Added<Torpedo>>,
    new_towers: Query<&TowerType, Added<Tower>>,
    dives: Query<&Trim, (With<UBoat>, Changed<Trim>)>,
    mut hits: MessageReader<Damage>,
    mut lost: MessageReader<TorpedoLost>,
    mut upgrades: MessageReader<UpgradeTower>,
    mut sells: MessageReader<SellTower>,
) {
    log.seconds += time.delta_secs();
    let now = log.seconds;

    let fired = new_torpedoes.iter().count() as u32;
    if fired > 0 {
        log.torpedoes_fired += fired;
        log.first_torpedo_at.get_or_insert(now);
    }
    for kind in &new_towers {
        let slot = TowerKind::ALL
            .iter()
            .position(|k| *k == kind.0)
            .unwrap_or(0);
        log.turrets_built[slot] += 1;
        log.first_build_at.get_or_insert(now);
    }
    if dives.iter().any(|t| matches!(t, Trim::Submerging(_))) {
        log.first_dive_at.get_or_insert(now);
    }
    log.torpedoes_hit += hits.read().filter(|d| d.weapon == Weapon::Torpedo).count() as u32;
    log.torpedoes_lost += lost.read().count() as u32;
    log.upgrades += upgrades.read().count() as u32;
    log.sells += sells.read().count() as u32;
}

fn on_sunk(
    sunk: On<Sunk>,
    mut log: ResMut<RunLog>,
    boats: Query<(), With<UBoat>>,
    enemies: Query<Option<&LastHitBy>, With<Enemy>>,
) {
    if boats.get(sunk.entity).is_ok() {
        log.boat_sunk += 1;
        return;
    }
    if let Ok(last) = enemies.get(sunk.entity) {
        log.kills += 1;
        match last.map(|l| l.0) {
            Some(Weapon::Ram) => log.ram_kills += 1,
            Some(Weapon::Blast) => log.chain_kills += 1,
            _ => {}
        }
    }
}

fn write_row(
    log: Res<RunLog>,
    wave: Res<WaveState>,
    treasury: Res<Treasury>,
    integrity: Res<Integrity>,
) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let won = wave.index >= WAVES.len();
    let header = "stamp,seconds,won,wave_reached,integrity,rm_end,torpedoes_fired,torpedoes_hit,torpedoes_lost,first_torpedo_s,first_build_s,first_dive_s,flak,gun88,heavy15,upgrades,sells,boat_sunk,kills,ram_kills,chain_kills";
    let opt = |v: Option<f32>| v.map_or(String::new(), |s| format!("{s:.1}"));
    let row = format!(
        "{stamp},{:.1},{won},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        log.seconds,
        wave.index,
        integrity.0,
        treasury.marks,
        log.torpedoes_fired,
        log.torpedoes_hit,
        log.torpedoes_lost,
        opt(log.first_torpedo_at),
        opt(log.first_build_at),
        opt(log.first_dive_at),
        log.turrets_built[0],
        log.turrets_built[1],
        log.turrets_built[2],
        log.upgrades,
        log.sells,
        log.boat_sunk,
        log.kills,
        log.ram_kills,
        log.chain_kills,
    );

    let result = fs::create_dir_all("telemetry").and_then(|_| {
        let mut file = fs::File::create(format!("telemetry/run-{stamp}.csv"))?;
        writeln!(file, "{header}")?;
        writeln!(file, "{row}")
    });
    if let Err(error) = result {
        warn!("telemetry not written: {error}");
    }
}
