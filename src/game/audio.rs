//! Sound: one-shots for what happens on the river and in the menus, a torpedo-run loop, and
//! two music layers that cross-fade with the waves; `M` mutes. A file under `assets/audio/`
//! plays, a missing one is silence. Sounds, levels, lengths: docs/AUDIO.md.
//! Sole writer of `Sfx`, `Mix`, `Voice`, `Layer`, `RunLoop`.

use std::collections::HashMap;

use bevy::audio::{AudioSinkPlayback, PlaybackMode, Volume};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::balance::{SizeClass, TowerKind, WAVES};
use crate::game::art::exists;
use crate::game::combat::{Damage, DamageKind, Fired, LastHitBy, Sunk, Weapon};
use crate::game::destruction::{style, Landed, Rng, Style};
use crate::game::enemy::{Enemy, EnemyReachedBunker, EnemyType, RangingStarted};
use crate::game::input::{
    BuildTower, CycleTargeting, Hovered, Refused, SellTower, ToggleMute, ToggleOverlay,
    TogglePause, UpgradeBoat, UpgradeTower,
};
use crate::game::map::RiverPath;
use crate::game::projectile::{Torpedo, TorpedoLost};
use crate::game::radial::Menu;
use crate::game::tower::{Constructing, Tier, Tower};
use crate::game::uboat::{Trim, UBoat};
use crate::game::wave::{WaveFinished, WaveStarted, WaveState};
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Sfx>()
        .init_resource::<Mix>()
        .add_systems(Startup, start_music)
        .add_systems(OnEnter(Screen::Gameplay), |mut s: Speaker| {
            s.play("prologue")
        })
        .add_systems(OnEnter(Screen::Victory), |mut s: Speaker| s.play("victory"))
        .add_systems(OnEnter(Screen::Defeat), |mut s: Speaker| s.play("defeat"))
        .add_systems(
            Update,
            (
                toggle_mute,
                (on_fired, on_damage, on_lost, on_landed, on_warnings),
                (on_trim, on_towers, on_menu, on_waves),
                torpedo_run,
                fade_music,
            )
                .chain(),
        )
        .add_observer(on_sunk);
}

/// One sound as `docs/AUDIO.md` lists it: level in dB (0 = the loudest sound), pitch spread
/// as a fraction, voices allowed at once, and the shortest gap in seconds between two starts.
struct Spec {
    key: &'static str,
    level: f32,
    pitch: f32,
    voices: usize,
    gap: f32,
}

const fn spec(key: &'static str, level: f32, pitch: f32, voices: usize, gap: f32) -> Spec {
    Spec {
        key,
        level,
        pitch,
        voices,
        gap,
    }
}

const SOUNDS: &[Spec] = &[
    spec("flak_shot", -14.0, 0.08, 4, 0.05),
    spec("gun88_shot", -10.0, 0.06, 3, 0.08),
    spec("heavy15_shot", -6.0, 0.04, 2, 0.15),
    spec("deck_gun", -10.0, 0.06, 2, 0.1),
    spec("enemy_gun", -12.0, 0.06, 2, 0.1),
    spec("ranging", -6.0, 0.0, 1, 1.0),
    spec("torpedo_launch", -8.0, 0.05, 2, 0.1),
    spec("hit", -16.0, 0.10, 4, 0.04),
    spec("torpedo_hit", -3.0, 0.04, 2, 0.1),
    spec("explode_small", -8.0, 0.08, 3, 0.05),
    spec("explode_medium", -5.0, 0.06, 3, 0.08),
    spec("explode_large", 0.0, 0.05, 2, 0.12),
    spec("bank_hit", -6.0, 0.06, 2, 0.1),
    spec("splash", -16.0, 0.10, 3, 0.05),
    spec("debris_land", -16.0, 0.10, 3, 0.05),
    spec("burn", -14.0, 0.06, 2, 0.1),
    spec("sink", -12.0, 0.06, 2, 0.2),
    spec("breach", -4.0, 0.0, 1, 0.3),
    spec("dive", -8.0, 0.0, 1, 0.3),
    spec("surface", -10.0, 0.0, 1, 0.3),
    spec("boat_hit", -8.0, 0.06, 2, 0.1),
    spec("ram", -8.0, 0.06, 2, 0.1),
    spec("boat_lost", -4.0, 0.0, 1, 1.0),
    spec("build", -12.0, 0.06, 2, 0.1),
    spec("built", -10.0, 0.04, 2, 0.1),
    spec("upgrade", -10.0, 0.04, 1, 0.1),
    spec("sell", -10.0, 0.04, 1, 0.1),
    spec("too_expensive", -14.0, 0.0, 1, 0.2),
    spec("coin", -20.0, 0.10, 2, 0.08),
    spec("ui_hover", -22.0, 0.10, 2, 0.03),
    spec("ui_click", -16.0, 0.06, 2, 0.03),
    spec("menu_open", -18.0, 0.06, 1, 0.05),
    spec("menu_close", -20.0, 0.06, 1, 0.05),
    spec("overlay", -16.0, 0.04, 1, 0.1),
    spec("pause", -14.0, 0.0, 1, 0.1),
    spec("prologue", -8.0, 0.0, 1, 1.0),
    spec("wave_start", -4.0, 0.0, 1, 1.0),
    spec("wave_clear", -6.0, 0.0, 1, 1.0),
    spec("last_wave", -6.0, 0.0, 1, 1.0),
    spec("victory", -4.0, 0.0, 1, 1.0),
    spec("defeat", -4.0, 0.0, 1, 1.0),
];

const RUN_LEVEL: f32 = -18.0;
/// Build and wave layer levels in dB, and seconds for a cross-fade.
const MUSIC_LEVELS: [f32; 2] = [-18.0, -16.0];
const MUSIC_FADE: f32 = 1.0;
/// Variant files looked for: `name.ogg`, then `name_1.ogg` up to this.
const MAX_VARIANTS: usize = 8;

/// Every clip that exists on disk, by sound key.
#[derive(Resource)]
pub struct Sfx {
    clips: HashMap<&'static str, Vec<Handle<AudioSource>>>,
    run: Option<Handle<AudioSource>>,
    music: [Option<Handle<AudioSource>>; 2],
}

impl FromWorld for Sfx {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let load = |key: String| exists(&key).then(|| assets.load(key));
        let clips: HashMap<_, Vec<_>> = SOUNDS
            .iter()
            .map(|s| {
                let files = std::iter::once(format!("audio/sfx/{}.ogg", s.key))
                    .chain((1..=MAX_VARIANTS).map(|i| format!("audio/sfx/{}_{i}.ogg", s.key)));
                (s.key, files.filter_map(load).collect())
            })
            .collect();
        let found = clips.values().filter(|v| !v.is_empty()).count();
        let sfx = Self {
            clips,
            run: load("audio/sfx/torpedo_run.ogg".into()),
            music: [
                load("audio/music/build.ogg".into()),
                load("audio/music/wave.ogg".into()),
            ],
        };
        info!(
            "audio: {found} of {} sounds, music: {}",
            SOUNDS.len(),
            sfx.music.iter().flatten().count()
        );
        sfx
    }
}

/// Mute switch, last start per sound, and the dice for variants and pitch.
#[derive(Resource, Default)]
pub struct Mix {
    muted: bool,
    last: HashMap<&'static str, f32>,
    rng: Rng,
}

/// A playing one-shot, counted against its sound's voice limit.
#[derive(Component)]
pub struct Voice(&'static str);

/// One music layer: 0 build, 1 wave. `gain` is linear, 0 to 1.
#[derive(Component)]
pub struct Layer {
    index: usize,
    gain: f32,
}

#[derive(Component)]
pub struct RunLoop;

#[derive(SystemParam)]
pub struct Speaker<'w, 's> {
    commands: Commands<'w, 's>,
    sfx: Res<'w, Sfx>,
    mix: ResMut<'w, Mix>,
    time: Res<'w, Time<Real>>,
    voices: Query<'w, 's, &'static Voice>,
}

impl Speaker<'_, '_> {
    /// Plays `key` unless muted, missing, too soon after the last start, or at its voice limit.
    pub fn play(&mut self, key: &'static str) {
        let Some(spec) = SOUNDS.iter().find(|s| s.key == key) else {
            return;
        };
        let Some(clips) = self.sfx.clips.get(key).filter(|c| !c.is_empty()) else {
            return;
        };
        let now = self.time.elapsed_secs();
        if self.mix.muted || self.mix.last.get(key).is_some_and(|&t| now - t < spec.gap) {
            return;
        }
        if self.voices.iter().filter(|v| v.0 == key).count() >= spec.voices {
            return;
        }
        self.mix.last.insert(key, now);
        let clip = clips[(self.mix.rng.unit() * clips.len() as f32) as usize % clips.len()].clone();
        let speed = 1.0 + self.mix.rng.range(-spec.pitch, spec.pitch);
        self.commands.spawn((
            Name::new("Sound"),
            Voice(key),
            AudioPlayer::new(clip),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Decibels(spec.level),
                speed,
                ..default()
            },
        ));
    }
}

fn toggle_mute(
    mut intents: MessageReader<ToggleMute>,
    mut mix: ResMut<Mix>,
    mut commands: Commands,
    voices: Query<Entity, Or<(With<Voice>, With<RunLoop>)>>,
) {
    if intents.read().count().is_multiple_of(2) {
        return;
    }
    mix.muted = !mix.muted;
    if mix.muted {
        for entity in &voices {
            commands.entity(entity).despawn();
        }
    }
}

fn on_fired(mut fired: MessageReader<Fired>, mut speaker: Speaker) {
    for shot in fired.read() {
        speaker.play(match shot.weapon {
            Weapon::Turret(TowerKind::Flak) => "flak_shot",
            Weapon::Turret(TowerKind::Gun88) => "gun88_shot",
            Weapon::Turret(TowerKind::Heavy15) => "heavy15_shot",
            Weapon::DeckGun => "deck_gun",
            Weapon::Torpedo => "torpedo_launch",
            Weapon::Enemy | Weapon::Ram | Weapon::Blast => "enemy_gun",
        });
    }
}

fn on_damage(mut hits: MessageReader<Damage>, mut speaker: Speaker) {
    for hit in hits.read() {
        speaker.play(match (hit.kind, hit.weapon) {
            (DamageKind::Shell, Weapon::Enemy) => "boat_hit",
            (DamageKind::Shell, _) => "hit",
            (DamageKind::Torpedo, _) => "torpedo_hit",
            (DamageKind::Ram, _) => "ram",
            (DamageKind::Blast, _) => continue,
        });
    }
}

fn on_lost(mut lost: MessageReader<TorpedoLost>, path: Res<RiverPath>, mut speaker: Speaker) {
    for lost in lost.read() {
        speaker.play(if path.contains(lost.at) {
            "splash"
        } else {
            "bank_hit"
        });
    }
}

fn on_landed(mut landed: MessageReader<Landed>, mut speaker: Speaker) {
    for piece in landed.read() {
        speaker.play(if piece.water { "splash" } else { "debris_land" });
    }
}

fn on_warnings(
    mut ranging: MessageReader<RangingStarted>,
    mut breaches: MessageReader<EnemyReachedBunker>,
    mut speaker: Speaker,
) {
    if ranging.read().count() > 0 {
        speaker.play("ranging");
    }
    if breaches.read().count() > 0 {
        speaker.play("breach");
    }
}

fn on_sunk(
    sunk: On<Sunk>,
    ships: Query<(&EnemyType, Option<&LastHitBy>), With<Enemy>>,
    boats: Query<(), With<UBoat>>,
    mut speaker: Speaker,
) {
    if boats.contains(sunk.entity) {
        speaker.play("boat_lost");
        return;
    }
    let Ok((kind, last_hit)) = ships.get(sunk.entity) else {
        return;
    };
    let class = kind.0.stats().class;
    let big = matches!(class, SizeClass::Heavy | SizeClass::Capital);
    match style(class, last_hit.map_or(Weapon::Enemy, |h| h.0)) {
        Style::Burn => speaker.play("burn"),
        Style::Explode if class == SizeClass::Light => speaker.play("explode_small"),
        Style::Explode if !big => speaker.play("explode_medium"),
        Style::Break if !big => speaker.play("explode_medium"),
        Style::Explode | Style::Break => speaker.play("explode_large"),
        Style::Capsize => speaker.play("sink"),
    }
    speaker.play("coin");
}

/// The start of a dive or of surfacing, not every tick of it.
fn on_trim(boats: Query<&Trim, With<UBoat>>, mut last: Local<u8>, mut speaker: Speaker) {
    let Ok(trim) = boats.single() else {
        return;
    };
    let now = match trim {
        Trim::Surfaced => 0,
        Trim::Submerging(_) => 1,
        Trim::Submerged => 2,
        Trim::Surfacing(_) => 3,
    };
    if now != *last {
        match now {
            1 => speaker.play("dive"),
            3 => speaker.play("surface"),
            _ => {}
        }
        *last = now;
    }
}

fn on_towers(
    added: Query<(), Added<Tower>>,
    upgraded: Query<(), (Changed<Tier>, Without<Constructing>)>,
    mut built: RemovedComponents<Constructing>,
    mut sold: MessageReader<SellTower>,
    mut refused: MessageReader<Refused>,
    mut speaker: Speaker,
) {
    if !added.is_empty() {
        speaker.play("build");
    }
    if built.read().count() > 0 {
        speaker.play("built");
    }
    if !upgraded.is_empty() {
        speaker.play("upgrade");
    }
    if sold.read().count() > 0 {
        speaker.play("sell");
    }
    if refused.read().count() > 0 {
        speaker.play("too_expensive");
    }
}

#[derive(SystemParam)]
struct MenuIntents<'w, 's> {
    build: MessageReader<'w, 's, BuildTower>,
    upgrade: MessageReader<'w, 's, UpgradeTower>,
    cycle: MessageReader<'w, 's, CycleTargeting>,
    boat: MessageReader<'w, 's, UpgradeBoat>,
    overlay: MessageReader<'w, 's, ToggleOverlay>,
    pause: MessageReader<'w, 's, TogglePause>,
}

fn on_menu(
    menu: Res<Menu>,
    hovered: Res<Hovered>,
    mut was_open: Local<bool>,
    mut intents: MenuIntents,
    mut speaker: Speaker,
) {
    if menu.open.is_some() != *was_open {
        *was_open = menu.open.is_some();
        speaker.play(if *was_open { "menu_open" } else { "menu_close" });
    }
    if hovered.is_changed() && hovered.0.is_some() {
        speaker.play("ui_hover");
    }
    let clicks = intents.build.read().count()
        + intents.upgrade.read().count()
        + intents.cycle.read().count()
        + intents.boat.read().count();
    if clicks > 0 {
        speaker.play("ui_click");
    }
    if intents.overlay.read().count() > 0 {
        speaker.play("overlay");
    }
    if intents.pause.read().count() > 0 {
        speaker.play("pause");
    }
}

fn on_waves(
    mut started: MessageReader<WaveStarted>,
    mut finished: MessageReader<WaveFinished>,
    mut speaker: Speaker,
) {
    for wave in started.read() {
        speaker.play(if wave.index + 1 == WAVES.len() {
            "last_wave"
        } else {
            "wave_start"
        });
    }
    if finished.read().count() > 0 {
        speaker.play("wave_clear");
    }
}

/// One quiet loop while any torpedo runs, however many.
fn torpedo_run(
    mut commands: Commands,
    sfx: Res<Sfx>,
    mix: Res<Mix>,
    torpedoes: Query<(), With<Torpedo>>,
    loops: Query<Entity, With<RunLoop>>,
) {
    let running = !torpedoes.is_empty() && !mix.muted;
    match (running, loops.single(), sfx.run.as_ref()) {
        (true, Err(_), Some(clip)) => {
            commands.spawn((
                Name::new("Torpedo run"),
                RunLoop,
                AudioPlayer::new(clip.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Decibels(RUN_LEVEL)),
            ));
        }
        (false, Ok(entity), _) => commands.entity(entity).despawn(),
        _ => {}
    }
}

/// Both layers play from the start, in step; only their gains move.
fn start_music(mut commands: Commands, sfx: Res<Sfx>) {
    for (index, clip) in sfx.music.iter().enumerate() {
        if let Some(clip) = clip {
            commands.spawn((
                Name::new("Music"),
                Layer { index, gain: 0.0 },
                AudioPlayer::new(clip.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::SILENT),
            ));
        }
    }
}

fn fade_music(
    time: Res<Time<Real>>,
    mix: Res<Mix>,
    wave: Option<Res<WaveState>>,
    mut layers: Query<(&mut Layer, &mut AudioSink)>,
) {
    let fighting = wave.is_some_and(|w| w.running);
    let step = time.delta_secs() / MUSIC_FADE;
    for (mut layer, mut sink) in &mut layers {
        let wanted = if mix.muted || (layer.index == 1) != fighting {
            0.0
        } else {
            1.0
        };
        layer.gain += (wanted - layer.gain).clamp(-step, step);
        let full = Volume::Decibels(MUSIC_LEVELS[layer.index]).to_linear();
        sink.set_volume(Volume::Linear(full * layer.gain));
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    fn world() -> World {
        let mut world = World::new();
        world.insert_resource(Sfx {
            clips: HashMap::from([("hit", vec![Handle::default()])]),
            run: None,
            music: [None, None],
        });
        world.init_resource::<Mix>();
        world.init_resource::<Time<Real>>();
        world
    }

    fn voices(world: &mut World) -> usize {
        world.query::<&Voice>().iter(world).count()
    }

    #[test]
    fn a_sound_at_its_voice_limit_starts_no_more() {
        let mut world = world();
        for _ in 0..4 {
            world.spawn(Voice("hit"));
        }
        world
            .run_system_once(|mut s: Speaker| s.play("hit"))
            .unwrap();
        assert_eq!(voices(&mut world), 4);
    }

    #[test]
    fn the_same_sound_twice_within_its_gap_plays_once() {
        let mut world = world();
        world
            .run_system_once(|mut s: Speaker| {
                s.play("hit");
                s.play("hit");
            })
            .unwrap();
        assert_eq!(voices(&mut world), 1);
    }

    #[test]
    fn a_missing_file_is_silence() {
        let mut world = world();
        world
            .run_system_once(|mut s: Speaker| s.play("explode_large"))
            .unwrap();
        assert_eq!(voices(&mut world), 0);
    }
}
