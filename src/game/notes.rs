//! Onboarding by showing, not telling: a clicking mouse on the water until the boat first
//! moves, a pressed Space key beside the boat while a target sits in its torpedo lane until
//! the first shot, pulsing build spots before the first build. Short pencil notes at the
//! place of action, each hint once per run, and feedback that repeats ("too expensive").
//! Onboarding follows the hints setting. Other plugins write `Hint`. Sole writer of `Notes`,
//! `Note`, `Gesture`. Rules: docs/design/onboarding.md

use std::collections::HashSet;

use bevy::prelude::*;

use crate::balance::{
    BUILD_SPOT_RADIUS, MAP_HEIGHT_TILES, MAP_WIDTH_TILES, NOTE_SECONDS, TILE_SIZE, Z_NOTE,
};
use crate::game::art::{Art, PaperTag};
use crate::game::combat::{Fired, Weapon};
use crate::game::enemy::Enemy;
use crate::game::input::MoveUBoat;
use crate::game::map::{BuildSpot, RiverPath};
use crate::game::palette::{INK, RED};
use crate::game::projectile::TorpedoLost;
use crate::game::stroke::Strokes;
use crate::game::tower::{Tier, Tower};
use crate::game::uboat::{Heading, Lane, Station, Torpedoes, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;
use crate::settings::Settings;

pub fn plugin(app: &mut App) {
    app.init_resource::<Notes>()
        .add_message::<Hint>()
        .add_message::<ShowNote>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            Update,
            (
                learn, gestures, prep_hints, bank_hints, show_hints, fade_notes,
            )
                .chain()
                .in_set(Frame::Present),
        );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HintKey {
    BankHit,
    EmptyTubes,
    BowNotAligned,
    ContactAbove,
    Upgrade,
}

impl HintKey {
    fn text(self) -> &'static str {
        match self {
            HintKey::BankHit => "Nur geradeaus!",
            HintKey::EmptyTubes => "Rohre leer: zum Bunker",
            HintKey::BowNotAligned => "Bug ausrichten",
            HintKey::ContactAbove => "Kontakt über uns!",
            HintKey::Upgrade => "Ausbauen?",
        }
    }
}

/// A hint at a world position. Shown once per run per key while hints are on.
#[derive(Message)]
pub struct Hint {
    pub key: HintKey,
    pub at: Vec2,
}

/// Feedback that repeats every time, e.g. "too expensive". Independent of the hints setting.
#[derive(Message)]
pub struct ShowNote {
    pub text: &'static str,
    pub at: Vec2,
}

/// What this run has shown and what the player has done already.
#[derive(Resource, Default)]
pub struct Notes {
    shown: HashSet<HintKey>,
    /// Seconds spent in the prologue.
    prologue_seconds: f32,
    moved: bool,
    fired: bool,
}

#[derive(Component)]
struct Note(Timer);

/// A drawing that shows what to do, flipping between its two frames.
#[derive(Component)]
struct Gesture(Show);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Show {
    /// Click the water to steer.
    Click,
    /// Space fires.
    Fire,
}

impl Show {
    fn frames(self) -> &'static str {
        match self {
            Show::Click => "ui/hints/mouse",
            Show::Fire => "ui/hints/space",
        }
    }

    /// World units on the map: the drawn size (docs/ART.md) blown up, a one-time lesson
    /// has to be seen.
    fn size(self) -> Vec2 {
        GESTURE_SCALE
            * match self {
                Show::Click => Vec2::new(32.0, 40.0),
                Show::Fire => Vec2::new(72.0, 24.0),
            }
    }

    /// The words standing in until the drawing exists.
    fn stand_in(self) -> &'static str {
        match self {
            Show::Click => "Klick",
            Show::Fire => "Leertaste",
        }
    }

    fn caption(self) -> Option<&'static str> {
        match self {
            Show::Click => None,
            Show::Fire => Some("Torpedo!"),
        }
    }
}

/// Seconds into the prologue before the mouse shows: the banner goes first.
const CLICK_DELAY: f32 = 1.0;
/// How far upstream of the boat the mouse clicks.
const CLICK_AHEAD: f32 = 6.0 * TILE_SIZE;
/// How far beside the boat the Space key sits.
const FIRE_BESIDE: f32 = 2.2 * TILE_SIZE;
/// Real seconds per frame of a gesture; it also swells a little on every press.
const GESTURE_FRAME: f32 = 0.4;
const GESTURE_SCALE: f32 = 2.5;
const GESTURE_FONT: f32 = 34.0;

fn reset(mut notes: ResMut<Notes>) {
    *notes = Notes::default();
}

fn learn(
    time: Res<Time>,
    wave: Res<WaveState>,
    mut moved: MessageReader<MoveUBoat>,
    mut fired: MessageReader<Fired>,
    mut notes: ResMut<Notes>,
) {
    if wave.prologue {
        notes.prologue_seconds += time.delta_secs();
    }
    if moved.read().count() > 0 {
        notes.moved = true;
    }
    if fired.read().any(|shot| shot.weapon == Weapon::Torpedo) {
        notes.fired = true;
    }
}

fn gestures(
    mut commands: Commands,
    time: Res<Time<Real>>,
    art: Res<Art>,
    settings: Res<Settings>,
    notes: Res<Notes>,
    wave: Res<WaveState>,
    path: Res<RiverPath>,
    boats: Query<(&Transform, &Station, &Heading, &Torpedoes), With<UBoat>>,
    enemies: Query<&Transform, (With<Enemy>, Without<UBoat>)>,
    mut shown: Query<
        (Entity, &Gesture, &mut Transform, Option<&mut Sprite>),
        (Without<UBoat>, Without<Enemy>),
    >,
) {
    let boat = boats.single().ok().filter(|_| settings.hints);
    let click = boat
        .filter(|_| wave.prologue && notes.prologue_seconds > CLICK_DELAY && !notes.moved)
        .map(|(_, station, ..)| path.point_at(station.along - CLICK_AHEAD));
    let fire =
        boat.filter(|_| !notes.fired)
            .and_then(|(transform, station, heading, torpedoes)| {
                let lane = Lane::new(&path, transform.translation.truncate(), station, heading);
                let target = enemies
                    .iter()
                    .any(|e| lane.contains(e.translation.truncate()));
                (lane.aligned && torpedoes.loaded > 0 && target).then(|| {
                    let side = lane.axis.perp();
                    let side = if side.y >= 0.0 { side } else { -side };
                    lane.from + side * FIRE_BESIDE
                })
            });

    let frame = (time.elapsed_secs() / GESTURE_FRAME) as usize % 2;
    for (show, wanted) in [(Show::Click, click), (Show::Fire, fire)] {
        let wanted = wanted.map(|at| on_map(show, at));
        let current = shown.iter_mut().find(|(_, g, ..)| g.0 == show);
        match (wanted, current) {
            (Some(at), Some((_, _, mut transform, sprite))) => {
                transform.translation = at.extend(Z_NOTE);
                transform.scale = Vec3::splat(if frame == 1 { 1.1 } else { 1.0 });
                let frames = art.frames(show.frames(), 2);
                if let (Some(mut sprite), Some(image)) = (sprite, frames.get(frame)) {
                    if sprite.image != *image {
                        sprite.image = image.clone();
                    }
                }
            }
            (Some(at), None) => spawn_gesture(&mut commands, &art, show, at),
            (None, Some((entity, ..))) => commands.entity(entity).despawn(),
            (None, None) => {}
        }
    }
}

/// `at`, moved just enough that the drawing and its caption stay on the map.
fn on_map(show: Show, at: Vec2) -> Vec2 {
    let half_map = Vec2::new(MAP_WIDTH_TILES, MAP_HEIGHT_TILES) * TILE_SIZE * 0.5;
    let half = show.size() * 0.5 + Vec2::splat(TILE_SIZE * 0.3);
    let below = if show.caption().is_some() {
        GESTURE_FONT * 1.6
    } else {
        0.0
    };
    Vec2::new(
        at.x.clamp(-half_map.x + half.x, half_map.x - half.x),
        at.y.clamp(-half_map.y + half.y + below, half_map.y - half.y),
    )
}

fn spawn_gesture(commands: &mut Commands, art: &Art, show: Show, at: Vec2) {
    let mut gesture = commands.spawn((
        Name::new("Gesture"),
        Gesture(show),
        DespawnOnExit(Screen::Gameplay),
        Transform::from_translation(at.extend(Z_NOTE)),
        Visibility::default(),
    ));
    match art.frames(show.frames(), 2).first() {
        Some(image) => gesture.insert(Sprite {
            image: image.clone(),
            custom_size: Some(show.size()),
            ..default()
        }),
        None => gesture.insert((
            Text2d::new(show.stand_in()),
            TextFont {
                font_size: FontSize::Px(GESTURE_FONT),
                ..default()
            },
            TextColor(INK),
            PaperTag,
        )),
    };
    let gesture = gesture.id();
    if let Some(caption) = show.caption() {
        commands.spawn((
            Text2d::new(caption),
            TextFont {
                font_size: FontSize::Px(GESTURE_FONT),
                ..default()
            },
            TextColor(RED),
            PaperTag,
            Transform::from_xyz(0.0, -(show.size().y * 0.5 + GESTURE_FONT * 0.7), 0.5),
            ChildOf(gesture),
        ));
    }
}

/// Wave 1 prep: every spot pulses until the first turret stands. Wave 3 prep: one turret
/// asks to be upgraded.
fn prep_hints(
    mut strokes: ResMut<Strokes>,
    time: Res<Time>,
    settings: Res<Settings>,
    wave: Res<WaveState>,
    mut hints: MessageWriter<Hint>,
    spots: Query<&Transform, With<BuildSpot>>,
    towers: Query<(&Transform, &Tier), With<Tower>>,
) {
    if !settings.hints || wave.prologue || wave.running {
        return;
    }
    if wave.index == 0 && towers.is_empty() {
        let pulse = 1.0 + 0.25 * (time.elapsed_secs() * 4.0).sin();
        for transform in &spots {
            let at = transform.translation.truncate();
            strokes.circle(at, BUILD_SPOT_RADIUS * pulse, RED);
        }
    }
    if wave.index == 2 {
        if let Some((transform, _)) = towers.iter().find(|(_, tier)| tier.0 == 0) {
            hints.write(Hint {
                key: HintKey::Upgrade,
                at: transform.translation.truncate(),
            });
        }
    }
}

fn bank_hints(mut lost: MessageReader<TorpedoLost>, mut hints: MessageWriter<Hint>) {
    for lost in lost.read() {
        hints.write(Hint {
            key: HintKey::BankHit,
            at: lost.at,
        });
    }
}

fn show_hints(
    mut hints: MessageReader<Hint>,
    mut plain: MessageReader<ShowNote>,
    settings: Res<Settings>,
    mut notes: ResMut<Notes>,
    mut commands: Commands,
) {
    for hint in hints.read() {
        if !settings.hints || !notes.shown.insert(hint.key) {
            continue;
        }
        commands.spawn(note(hint.key.text(), hint.at, NOTE_SECONDS));
    }
    for plain in plain.read() {
        commands.spawn(note(plain.text, plain.at, 1.2));
    }
}

fn note(text: &'static str, at: Vec2, seconds: f32) -> impl Bundle {
    (
        Name::new("Note"),
        Note(Timer::from_seconds(seconds, TimerMode::Once)),
        DespawnOnExit(Screen::Gameplay),
        Text2d::new(text),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(INK),
        PaperTag,
        Transform::from_translation((at + Vec2::new(0.0, 32.0)).extend(Z_NOTE)),
    )
}

fn fade_notes(
    time: Res<Time>,
    mut commands: Commands,
    mut notes: Query<(Entity, &mut Note, &mut TextColor, &mut Transform)>,
) {
    for (entity, mut note, mut color, mut transform) in &mut notes {
        if note.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let left = note.0.fraction_remaining();
        color.0.set_alpha(left.min(1.0));
        transform.translation.y += 6.0 * time.delta_secs();
    }
}
