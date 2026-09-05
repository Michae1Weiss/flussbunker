//! Wave pacing: what arrives, when, and who won.
//!
//! Two pacing rules are encoded here, both of them design decisions rather
//! than technical ones:
//!
//! * **Waves never auto-start.** The player presses Space. A guaranteed
//!   decision window between waves is what separates "tower defense" from
//!   "watch a screensaver" — it is the moment the whole build-adapt loop
//!   actually happens.
//! * **The wave is over only when the river is empty**, not when the last boat
//!   has spawned. Otherwise the prep phase overlaps with combat and the player
//!   never gets the pause they were promised.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use crate::balance::{EnemyKind, WAVES};
use crate::game::enemy::{depth_charge_bundle, enemy_bundle, Enemy};
use crate::game::map::RiverPath;
use crate::game_plugin::Sim;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<WaveState>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            Update,
            start_wave
                .run_if(in_state(Screen::Gameplay).and(input_just_pressed(KeyCode::Space))),
        )
        .add_systems(FixedUpdate, (spawn_boats, finish_wave).chain().in_set(Sim::Spawn));
}

#[derive(Resource, Debug)]
pub struct WaveState {
    pub index: usize,
    pub running: bool,
    /// Seconds since the wave started; groups compare against their lead-in.
    elapsed: f32,
    /// How many of each group have already been put in the water.
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
        format!("{}/{} {}", self.index + 1, WAVES.len(), WAVES[self.index].name)
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

/// Each group is an independent little schedule: wait `lead_in`, then release
/// one boat every `spacing` seconds. Overlapping groups is what makes wave 5
/// feel like a fleet instead of a queue — and it falls out of the data with no
/// extra machinery.
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
    // Conditional extras go on afterwards. A bundle describes what is always
    // true; anything optional is an `insert`.
    if let Some(charges) = depth_charge_bundle(kind) {
        entity.insert(charges);
    }
}

fn finish_wave(
    mut wave: ResMut<WaveState>,
    mut next: ResMut<NextState<Screen>>,
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
    if wave.index >= WAVES.len() {
        next.set(Screen::Victory);
    }
}
