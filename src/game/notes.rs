//! Reactive onboarding notes: one-line pencil notes at the place of action, each hint
//! once per run. Other plugins write `Hint`; time-based prologue and prep hints are
//! derived here from public state. Sole writer of `Notes`, `Note`.
//! Rules: docs/design/onboarding.md

use std::collections::HashSet;

use bevy::prelude::*;

use crate::balance::{BUILD_SPOT_RADIUS, NOTE_SECONDS, Z_UI};
use crate::game::map::BuildSpot;
use crate::game::palette::{INK, NOTE, RED};
use crate::game::projectile::TorpedoLost;
use crate::game::tower::{Tier, Tower};
use crate::game::uboat::{Model, Station, Torpedoes, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<Notes>()
        .add_message::<Hint>()
        .add_message::<ShowNote>()
        .add_systems(OnEnter(Screen::Gameplay), reset)
        .add_systems(
            Update,
            (
                prologue_hints,
                prep_hints,
                bank_hints,
                show_hints,
                fade_notes,
            )
                .chain()
                .in_set(Frame::Present),
        );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HintKey {
    Move,
    Fire,
    BankHit,
    EmptyTubes,
    BowNotAligned,
    ContactAbove,
    Dive,
    Build,
    Upgrade,
}

impl HintKey {
    fn text(self) -> &'static str {
        match self {
            HintKey::Move => "Linksklick auf den Fluss: fahren",
            HintKey::Fire => "Space: Torpedo",
            HintKey::BankHit => "Torpedos laufen geradeaus. Kurven sind totes Wasser.",
            HintKey::EmptyTubes => "Rohre leer. Nachschub am Bunker.",
            HintKey::BowNotAligned => "Bug nicht ausgerichtet.",
            HintKey::ContactAbove => "Kontakt über uns!",
            HintKey::Dive => "Rechtsklick: tauchen",
            HintKey::Build => "Klick: Stellung bauen",
            HintKey::Upgrade => "Klick aufs Geschütz: ausbauen",
        }
    }
}

/// A hint at a world position. Shown once per run per key while notes are enabled.
#[derive(Message)]
pub struct Hint {
    pub key: HintKey,
    pub at: Vec2,
}

/// Feedback that repeats every time, e.g. "too expensive". Independent of the onboarding switch.
#[derive(Message)]
pub struct ShowNote {
    pub text: &'static str,
    pub at: Vec2,
}

#[derive(Resource)]
pub struct Notes {
    pub enabled: bool,
    shown: HashSet<HintKey>,
    /// Seconds spent in the prologue.
    prologue_seconds: f32,
}

impl Default for Notes {
    fn default() -> Self {
        Self {
            enabled: true,
            shown: HashSet::new(),
            prologue_seconds: 0.0,
        }
    }
}

#[derive(Component)]
struct Note(Timer);

fn reset(mut notes: ResMut<Notes>) {
    let enabled = notes.enabled;
    *notes = Notes {
        enabled,
        ..default()
    };
}

/// "Click the river" after a few idle seconds, "Space" after 20 s without a shot.
fn prologue_hints(
    time: Res<Time>,
    mut notes: ResMut<Notes>,
    wave: Res<WaveState>,
    mut hints: MessageWriter<Hint>,
    path: Res<crate::game::map::RiverPath>,
    boats: Query<(&Transform, &Station, &Model, &Torpedoes), With<UBoat>>,
) {
    if !wave.prologue {
        return;
    }
    notes.prologue_seconds += time.delta_secs();
    let Ok((transform, station, model, torpedoes)) = boats.single() else {
        return;
    };
    let at = transform.translation.truncate();
    if notes.prologue_seconds > 6.0 && station.is_docked(&path) {
        hints.write(Hint {
            key: HintKey::Move,
            at,
        });
    }
    if notes.prologue_seconds > 20.0 && torpedoes.loaded == model.0.stats().tubes {
        hints.write(Hint {
            key: HintKey::Fire,
            at,
        });
    }
}

/// Wave 1 prep: every spot pulses. Wave 3 prep: one turret asks to be upgraded.
fn prep_hints(
    mut gizmos: Gizmos,
    time: Res<Time>,
    notes: Res<Notes>,
    wave: Res<WaveState>,
    mut hints: MessageWriter<Hint>,
    spots: Query<&Transform, With<BuildSpot>>,
    towers: Query<(&Transform, &Tier), With<Tower>>,
) {
    if !notes.enabled || wave.prologue || wave.running {
        return;
    }
    if wave.index == 0 && towers.is_empty() {
        let pulse = 1.0 + 0.25 * (time.elapsed_secs() * 4.0).sin();
        for transform in &spots {
            let at = transform.translation.truncate();
            gizmos.circle_2d(
                Isometry2d::from_translation(at),
                BUILD_SPOT_RADIUS * pulse,
                RED,
            );
        }
        if let Some(transform) = spots.iter().next() {
            hints.write(Hint {
                key: HintKey::Build,
                at: transform.translation.truncate(),
            });
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
    mut notes: ResMut<Notes>,
    mut commands: Commands,
) {
    for hint in hints.read() {
        if !notes.enabled || !notes.shown.insert(hint.key) {
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
        TextBackgroundColor(NOTE),
        Transform::from_translation((at + Vec2::new(0.0, 32.0)).extend(Z_UI)),
    )
}

fn fade_notes(
    time: Res<Time>,
    mut commands: Commands,
    mut notes: Query<(
        Entity,
        &mut Note,
        &mut TextColor,
        &mut TextBackgroundColor,
        &mut Transform,
    )>,
) {
    for (entity, mut note, mut color, mut paper, mut transform) in &mut notes {
        if note.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let left = note.0.fraction_remaining();
        color.0.set_alpha(left.min(1.0));
        paper.0.set_alpha(left.min(1.0));
        transform.translation.y += 6.0 * time.delta_secs();
    }
}
