//! Run pacing: the prologue, then twelve waves spawned on their schedule, each ending
//! when the river is empty; victory after the last. Also the 1x/2x pace. Sole writer of
//! `WaveState` and of the virtual time speed; writes `WaveFinished`.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use crate::balance::{EnemyKind, WAVES, WAVE_CLEAR_BONUS};
use crate::game::economy::Treasury;
use crate::game::enemy::{enemy_bundle, shelling_bundle, Cruising, Enemy};
use crate::game::input::{TogglePause, ToggleSpeed};
use crate::game::map::RiverPath;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<WaveState>()
        .add_message::<WaveFinished>()
        .add_systems(OnEnter(Screen::Gameplay), (reset, spawn_prologue).chain())
        .add_systems(OnExit(Screen::Gameplay), normal_pace)
        .add_systems(
            Update,
            (
                start_wave.run_if(
                    in_state(Screen::Gameplay).and_then(input_just_pressed(KeyCode::Enter)),
                ),
                (toggle_pace, toggle_pause).run_if(in_state(Screen::Gameplay)),
            ),
        )
        .add_systems(
            FixedUpdate,
            (spawn_boats, finish_wave, finish_prologue)
                .chain()
                .in_set(Sim::Spawn),
        );
}

/// The river is empty and the prep phase begins.
#[derive(Message)]
pub struct WaveFinished;

#[derive(Resource, Debug)]
pub struct WaveState {
    /// Next wave to start, or the one running.
    pub index: usize,
    pub running: bool,
    /// The gold transport is still afloat; no waves and no building yet.
    pub prologue: bool,
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
            prologue: true,
            elapsed: 0.0,
            spawned: Vec::new(),
        }
    }
}

impl WaveState {
    pub fn label(&self) -> String {
        if self.prologue {
            return "Prolog: Goldtransport".to_string();
        }
        if self.index >= WAVES.len() {
            return "-".to_string();
        }
        format!(
            "{}/{} {}",
            self.index + 1,
            WAVES.len(),
            WAVES[self.index].name
        )
    }

    /// Composition of the next wave, e.g. "8 MTB, 4 Korvette".
    pub fn preview(&self) -> String {
        if self.prologue || self.running || self.index >= WAVES.len() {
            return String::new();
        }
        WAVES[self.index]
            .groups
            .iter()
            .map(|g| format!("{} {}", g.count, g.kind.stats().name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn reset(mut wave: ResMut<WaveState>, mut time: ResMut<Time<Virtual>>) {
    *wave = WaveState::default();
    time.set_relative_speed(1.0);
    time.unpause();
}

fn normal_pace(mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(1.0);
    time.unpause();
}

fn toggle_pause(mut intents: MessageReader<TogglePause>, mut time: ResMut<Time<Virtual>>) {
    for _ in intents.read() {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn spawn_prologue(mut commands: Commands, path: Res<RiverPath>) {
    commands.spawn((
        enemy_bundle(EnemyKind::GoldTransport, path.spawn_point()),
        Cruising::prologue(&path),
    ));
}

/// The pace toggles between 1x and 2x and always returns to 1x when a wave starts.
fn toggle_pace(mut intents: MessageReader<ToggleSpeed>, mut time: ResMut<Time<Virtual>>) {
    for _ in intents.read() {
        let next = if time.relative_speed() > 1.0 {
            1.0
        } else {
            2.0
        };
        time.set_relative_speed(next);
    }
}

fn start_wave(mut wave: ResMut<WaveState>, mut time: ResMut<Time<Virtual>>) {
    if wave.prologue || wave.running || wave.index >= WAVES.len() {
        return;
    }
    wave.running = true;
    wave.elapsed = 0.0;
    wave.spawned = vec![0; WAVES[wave.index].groups.len()];
    time.set_relative_speed(1.0);
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

fn finish_prologue(
    mut wave: ResMut<WaveState>,
    mut finished: MessageWriter<WaveFinished>,
    transports: Query<(), With<Cruising>>,
) {
    if wave.prologue && transports.is_empty() {
        wave.prologue = false;
        finished.write(WaveFinished);
    }
}
