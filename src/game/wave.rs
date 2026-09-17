//! Wave pacing: spawns groups on their schedule, ends a wave when the river is
//! empty, declares victory after the last one. Sole writer of `WaveState`; writes
//! `WaveFinished`.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use crate::balance::{EnemyKind, WAVES, WAVE_CLEAR_BONUS};
use crate::game::economy::Treasury;
use crate::game::enemy::{enemy_bundle, shelling_bundle, Enemy};
use crate::game::map::RiverPath;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<WaveState>()
        .add_message::<WaveFinished>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            Update,
            start_wave
                .run_if(in_state(Screen::Gameplay).and_then(input_just_pressed(KeyCode::Enter))),
        )
        .add_systems(
            FixedUpdate,
            (spawn_boats, finish_wave).chain().in_set(Sim::Spawn),
        );
}

/// The river is empty and the prep phase begins.
#[derive(Message)]
pub struct WaveFinished;

#[derive(Resource, Debug)]
pub struct WaveState {
    pub index: usize,
    pub running: bool,
    /// Seconds since the wave started.
    elapsed: f32,
    /// Per group, boats already spawned.
    spawned: Vec<u32>,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            index: 0,
            running: false,
            elapsed: 0.0,
            spawned: Vec::new(),
        }
    }
}

impl WaveState {
    pub fn label(&self) -> String {
        if self.index >= WAVES.len() {
            return "—".to_string();
        }
        format!(
            "{}/{} {}",
            self.index + 1,
            WAVES.len(),
            WAVES[self.index].name
        )
    }
}

fn reset(mut wave: ResMut<WaveState>) {
    *wave = WaveState::default();
}

fn start_wave(mut wave: ResMut<WaveState>) {
    if wave.running || wave.index >= WAVES.len() {
        return;
    }
    wave.running = true;
    wave.elapsed = 0.0;
    wave.spawned = vec![0; WAVES[wave.index].groups.len()];
}

fn spawn_boats(
    time: Res<Time>,
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    path: Res<RiverPath>,
) {
    if !wave.running || wave.index >= WAVES.len() {
        return;
    }
    wave.elapsed += time.delta_secs();

    let groups = WAVES[wave.index].groups;
    for (i, group) in groups.iter().enumerate() {
        let due = |n: u32| group.lead_in + n as f32 * group.spacing;
        while wave.spawned[i] < group.count && wave.elapsed >= due(wave.spawned[i]) {
            wave.spawned[i] += 1;
            spawn_one(&mut commands, group.kind, path.spawn_point());
        }
    }
}

fn spawn_one(commands: &mut Commands, kind: EnemyKind, at: Vec2) {
    let mut entity = commands.spawn(enemy_bundle(kind, at));
    if let Some(shelling) = shelling_bundle(kind) {
        entity.insert(shelling);
    }
}

fn finish_wave(
    mut wave: ResMut<WaveState>,
    mut next: ResMut<NextState<Screen>>,
    mut treasury: ResMut<Treasury>,
    mut finished: MessageWriter<WaveFinished>,
    enemies: Query<(), With<Enemy>>,
) {
    if !wave.running {
        return;
    }

    let all_spawned = WAVES[wave.index]
        .groups
        .iter()
        .zip(&wave.spawned)
        .all(|(group, spawned)| *spawned >= group.count);

    if !all_spawned || !enemies.is_empty() {
        return;
    }

    wave.running = false;
    wave.index += 1;
    treasury.marks += WAVE_CLEAR_BONUS;
    finished.write(WaveFinished);
    if wave.index >= WAVES.len() {
        next.set(Screen::Victory);
    }
}
