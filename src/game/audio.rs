//! Sound: one-shots for what happens on the river and in the menus, loops for running
//! torpedoes and the boat's engine, and two music layers that cross-fade with the waves;
//! volumes follow `Settings`, `M` mutes. A file under `assets/audio/` plays, a missing one is
//! silence. Sounds, levels, lengths: docs/AUDIO.md.
//! Sole writer of `Sfx`, `Mix`, `Voice`, `Layer`, `Loop`.

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
    BuildTower, CycleTargeting, Hovered, MoveUBoat, Pointing, Refused, SellTower, ToggleMute,
    ToggleOverlay, TogglePause, UpgradeBoat, UpgradeTower,
};
use crate::game::map::RiverPath;
use crate::game::projectile::{Torpedo, TorpedoLost};
use crate::game::radial::Menu;
use crate::game::tower::{Constructing, Tier, Tower};
use crate::game::uboat::{DryFire, Station, Torpedoes, Trim, UBoat};
use crate::game::wave::{WaveFinished, WaveStarted, WaveState};
use crate::screens::Screen;
use crate::settings::Settings;

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
                (on_trim, on_rearm, on_towers, on_menu, on_waves),
                loops,
                fade_music,
            )
                .chain(),
        )
        .add_observer(on_sunk);
}

/// One sound as `docs/AUDIO.md` lists it: level in dB (0 = the loudest sound), pitch spread
/// as a fraction, and the shortest gap in seconds between two starts.
struct Spec {
    key: &'static str,
    level: f32,
    pitch: f32,
    gap: f32,
}

const fn spec(key: &'static str, level: f32, pitch: f32, gap: f32) -> Spec {
    Spec {
        key,
        level,
        pitch,
        gap,
    }
}

/// Every event gets its sound: a gap only keeps one frame from stacking the same start, and
/// the long gaps belong to announcements that happen once.
const SOUNDS: &[Spec] = &[
    spec("flak_shot", -14.0, 0.08, 0.02),
    spec("gun88_shot", -10.0, 0.06, 0.02),
    spec("heavy15_shot", -6.0, 0.04, 0.02),
    spec("deck_gun", -10.0, 0.06, 0.02),
    spec("enemy_gun", -12.0, 0.06, 0.02),
    spec("ranging", -6.0, 0.0, 1.0),
    spec("torpedo_launch", -8.0, 0.05, 0.02),
    spec("torpedo_load", -14.0, 0.05, 0.02),
    spec("empty_tubes", -12.0, 0.0, 0.4),
    spec("hit", -16.0, 0.10, 0.02),
    spec("torpedo_hit", -3.0, 0.04, 0.02),
    spec("explode_small", -8.0, 0.08, 0.02),
    spec("explode_medium", -5.0, 0.06, 0.02),
    spec("explode_large", 0.0, 0.05, 0.02),
    spec("bank_hit", -6.0, 0.06, 0.02),
    spec("splash", -16.0, 0.10, 0.02),
    spec("debris_land", -16.0, 0.10, 0.02),
    spec("burn", -14.0, 0.06, 0.02),
    spec("sink", -12.0, 0.06, 0.02),
    spec("breach", -4.0, 0.0, 0.3),
    spec("dive", -8.0, 0.0, 0.3),
    spec("surface", -10.0, 0.0, 0.3),
    spec("boat_hit", -8.0, 0.06, 0.02),
    spec("ram", -8.0, 0.06, 0.02),
    spec("boat_lost", -4.0, 0.0, 1.0),
    spec("build", -12.0, 0.06, 0.02),
    spec("built", -10.0, 0.04, 0.02),
    spec("upgrade", -10.0, 0.04, 0.02),
    spec("sell", -10.0, 0.04, 0.02),
    spec("too_expensive", -14.0, 0.0, 0.1),
    spec("coin", -20.0, 0.10, 0.02),
    spec("ui_hover", -22.0, 0.10, 0.02),
    spec("ui_click", -16.0, 0.06, 0.02),
    spec("menu_open", -18.0, 0.06, 0.02),
    spec("menu_close", -20.0, 0.06, 0.02),
    spec("overlay", -16.0, 0.04, 0.1),
    spec("pause", -14.0, 0.0, 0.1),
    spec("prologue", -8.0, 0.0, 1.0),
    spec("wave_start", -4.0, 0.0, 1.0),
    spec("wave_clear", -6.0, 0.0, 1.0),
    spec("last_wave", -6.0, 0.0, 1.0),
    spec("victory", -4.0, 0.0, 1.0),
    spec("defeat", -4.0, 0.0, 1.0),
];

/// A sound not yet recorded plays the one beside it instead.
const STAND_INS: &[(&str, &str)] = &[
    ("burn", "explode_small"),
    ("torpedo_load", "built"),
    ("empty_tubes", "too_expensive"),
];

/// Copies of one sound at once; only a safety net, the stacking below keeps crowds quiet.
const VOICES: usize = 12;
/// dB less for each copy of the same sound already playing, so a chain of explosions swells
/// instead of clipping.
const STACK_DB: f32 = 2.0;

/// A sound that plays as long as something lasts, not once per event.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum Loop {
    /// While at least one torpedo runs, however many.
    TorpedoRun,
    /// While the boat moves or turns.
    Engine,
}

impl Loop {
    const ALL: [Loop; 2] = [Loop::TorpedoRun, Loop::Engine];

    fn key(self) -> &'static str {
        match self {
            Loop::TorpedoRun => "torpedo_run",
            Loop::Engine => "engine",
        }
    }

    fn level(self) -> f32 {
        match self {
            Loop::TorpedoRun => -18.0,
            Loop::Engine => -20.0,
        }
    }
}
/// Build and wave layer levels in dB, and seconds for a cross-fade.
const MUSIC_LEVELS: [f32; 2] = [-18.0, -16.0];
const MUSIC_FADE: f32 = 1.0;
/// Variant files looked for: `name.ogg`, then `name_1.ogg` up to this.
const MAX_VARIANTS: usize = 8;

/// Every clip that exists on disk, by sound key.
#[derive(Resource)]
pub struct Sfx {
    clips: HashMap<&'static str, Vec<Handle<AudioSource>>>,
    loops: Vec<(Loop, Handle<AudioSource>)>,
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
            loops: Loop::ALL
                .into_iter()
                .filter_map(|l| Some((l, load(format!("audio/sfx/{}.ogg", l.key()))?)))
                .collect(),
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

#[derive(SystemParam)]
pub struct Speaker<'w, 's> {
    commands: Commands<'w, 's>,
    sfx: Res<'w, Sfx>,
    mix: ResMut<'w, Mix>,
    settings: Res<'w, Settings>,
    time: Res<'w, Time<Real>>,
    voices: Query<'w, 's, &'static Voice>,
}

impl Speaker<'_, '_> {
    /// Plays `key` (or its stand-in while unrecorded) unless muted, missing, or too soon
    /// after its last start; each copy already playing makes it a little quieter.
    pub fn play(&mut self, key: &'static str) {
        let Some(spec) = SOUNDS.iter().find(|s| s.key == key) else {
            return;
        };
        let recorded = |k: &str| self.sfx.clips.get(k).filter(|c| !c.is_empty());
        let Some(clips) = recorded(key).or_else(|| {
            STAND_INS
                .iter()
                .find(|(k, _)| *k == key)
                .and_then(|(_, stand_in)| recorded(stand_in))
        }) else {
            return;
        };
        let now = self.time.elapsed_secs();
        let gain = Settings::gain(self.settings.sound);
        if self.mix.muted
            || gain == 0.0
            || self.mix.last.get(key).is_some_and(|&t| now - t < spec.gap)
        {
            return;
        }
        let playing = self.voices.iter().filter(|v| v.0 == key).count();
        if playing >= VOICES {
            return;
        }
        let clip = clips[(self.mix.rng.unit() * clips.len() as f32) as usize % clips.len()].clone();
        self.mix.last.insert(key, now);
        let speed = 1.0 + self.mix.rng.range(-spec.pitch, spec.pitch);
        let level = spec.level - STACK_DB * playing as f32;
        self.commands.spawn((
            Name::new("Sound"),
            Voice(key),
            AudioPlayer::new(clip),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(Volume::Decibels(level).to_linear() * gain),
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
    voices: Query<Entity, Or<(With<Voice>, With<Loop>)>>,
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

/// One clunk per torpedo loaded at the bunker (a new boat arriving full is not a load), a
/// dull knock when fire finds every tube empty.
fn on_rearm(
    boats: Query<(Entity, &Torpedoes), With<UBoat>>,
    mut dry: MessageReader<DryFire>,
    mut last: Local<Option<(Entity, u8)>>,
    mut speaker: Speaker,
) {
    if dry.read().count() > 0 {
        speaker.play("empty_tubes");
    }
    let Ok((boat, torpedoes)) = boats.single() else {
        return;
    };
    if *last == Some((boat, torpedoes.loaded.wrapping_sub(1))) {
        speaker.play("torpedo_load");
    }
    *last = Some((boat, torpedoes.loaded));
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
    steer: MessageReader<'w, 's, MoveUBoat>,
    overlay: MessageReader<'w, 's, ToggleOverlay>,
    pause: MessageReader<'w, 's, TogglePause>,
}

/// Taps as the cursor finds something to click (a radial entry, the water, a spot, a turret,
/// the docked boat), stamps when a click acts.
fn on_menu(
    menu: Res<Menu>,
    hovered: Res<Hovered>,
    pointing: Res<Pointing>,
    mut was_open: Local<bool>,
    mut intents: MenuIntents,
    mut speaker: Speaker,
) {
    if menu.open.is_some() != *was_open {
        *was_open = menu.open.is_some();
        speaker.play(if *was_open { "menu_open" } else { "menu_close" });
    }
    let entry = hovered.is_changed() && hovered.0.is_some();
    let ground = pointing.is_changed() && !matches!(*pointing, Pointing::Nothing | Pointing::Entry);
    if entry || ground {
        speaker.play("ui_hover");
    }
    let clicks = intents.build.read().count()
        + intents.upgrade.read().count()
        + intents.cycle.read().count()
        + intents.boat.read().count()
        + intents.steer.read().count();
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

/// Each loop plays while its cause lasts and the game is not paused; its volume follows the
/// sound setting.
fn loops(
    mut commands: Commands,
    sfx: Res<Sfx>,
    mix: Res<Mix>,
    settings: Res<Settings>,
    time: Res<Time<Virtual>>,
    torpedoes: Query<(), With<Torpedo>>,
    boats: Query<&Station, With<UBoat>>,
    mut playing: Query<(Entity, &Loop, Option<&mut AudioSink>)>,
) {
    let gain = if mix.muted {
        0.0
    } else {
        Settings::gain(settings.sound)
    };
    for (kind, clip) in &sfx.loops {
        let cause = match kind {
            Loop::TorpedoRun => !torpedoes.is_empty(),
            Loop::Engine => boats.iter().any(|s| s.goal != s.along),
        };
        let wanted = cause && gain > 0.0 && !time.is_paused();
        let volume = Volume::Linear(Volume::Decibels(kind.level()).to_linear() * gain);
        match (wanted, playing.iter_mut().find(|(_, l, _)| *l == kind)) {
            (true, None) => {
                commands.spawn((
                    Name::new("Loop"),
                    *kind,
                    AudioPlayer::new(clip.clone()),
                    PlaybackSettings::LOOP.with_volume(volume),
                ));
            }
            (true, Some((_, _, Some(mut sink)))) => sink.set_volume(volume),
            (false, Some((entity, _, _))) => commands.entity(entity).despawn(),
            _ => {}
        }
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
    settings: Res<Settings>,
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
        let full = Volume::Decibels(MUSIC_LEVELS[layer.index]).to_linear()
            * Settings::gain(settings.music);
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
            loops: Vec::new(),
            music: [None, None],
        });
        world.init_resource::<Mix>();
        world.init_resource::<Settings>();
        world.init_resource::<Time<Real>>();
        world
    }

    fn voices(world: &mut World) -> usize {
        world.query::<&Voice>().iter(world).count()
    }

    #[test]
    fn a_sound_at_its_voice_limit_starts_no_more() {
        let mut world = world();
        for _ in 0..VOICES {
            world.spawn(Voice("hit"));
        }
        world
            .run_system_once(|mut s: Speaker| s.play("hit"))
            .unwrap();
        assert_eq!(voices(&mut world), VOICES);
    }

    fn loudness(world: &mut World) -> Vec<f32> {
        world
            .query::<&PlaybackSettings>()
            .iter(world)
            .map(|p| p.volume.to_linear())
            .collect()
    }

    #[test]
    fn a_copy_of_a_playing_sound_is_quieter() {
        let mut world = world();
        world
            .run_system_once(|mut s: Speaker| s.play("hit"))
            .unwrap();
        let alone = loudness(&mut world)[0];
        world.resource_mut::<Mix>().last.clear();
        world.spawn(Voice("hit"));
        world.spawn(Voice("hit"));
        world
            .run_system_once(|mut s: Speaker| s.play("hit"))
            .unwrap();
        let crowded = loudness(&mut world).into_iter().fold(f32::MAX, f32::min);
        assert!(crowded < alone);
    }

    #[test]
    fn an_unrecorded_sound_plays_its_stand_in() {
        let mut world = world();
        world
            .resource_mut::<Sfx>()
            .clips
            .insert("built", vec![Handle::default()]);
        world
            .run_system_once(|mut s: Speaker| s.play("torpedo_load"))
            .unwrap();
        let keys: Vec<_> = world.query::<&Voice>().iter(&world).map(|v| v.0).collect();
        assert_eq!(keys, ["torpedo_load"]);
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
    fn sound_volume_zero_starts_nothing() {
        let mut world = world();
        world.resource_mut::<Settings>().sound = 0;
        world
            .run_system_once(|mut s: Speaker| s.play("hit"))
            .unwrap();
        assert_eq!(voices(&mut world), 0);
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
