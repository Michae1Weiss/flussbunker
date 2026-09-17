//! Three paper notes (RM with tweened value and floating transactions, wave, boat), the
//! Tab overlay (ranges, reloads, targeting, exact HP, torpedoes, route, shelling reach)
//! and enemy health bars. Reads other features' state; sole writer of `DisplayedMarks`,
//! `Overlay`, `OverlayLabel`. Rules: docs/design/ui-input.md

use bevy::prelude::*;

use crate::balance::{RM_TWEEN_SECONDS, TORPEDO_RUN, Z_UI};
use crate::game::bunker::Integrity;
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::{Enemy, Shelling};
use crate::game::input::ToggleOverlay;
use crate::game::map::RiverPath;
use crate::game::tower::{Gun, Range, Targeting, Tower};
use crate::game::uboat::{Flotilla, Heading, Station, Torpedoes, Trim, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<DisplayedMarks>()
        .init_resource::<Overlay>()
        .add_systems(OnEnter(Screen::Gameplay), (spawn_notes, reset_marks))
        .add_systems(
            Update,
            (
                toggle_overlay,
                tween_marks,
                update_rm_note,
                update_wave_note,
                update_boat_note,
                float_transactions,
                draw_health_bars,
                draw_route,
                draw_overlay,
                attach_labels,
                update_labels,
            )
                .chain()
                .in_set(Frame::Present),
        );
}

/// RM as shown: lags the treasury by `RM_TWEEN_SECONDS` and remembers the last real value
/// so a change can float up as a transaction.
#[derive(Resource, Default)]
pub struct DisplayedMarks {
    pub shown: f32,
    last: u32,
}

/// The tactical overlay, toggled with Tab. A mode, not a peek.
#[derive(Resource, Default)]
pub struct Overlay {
    pub on: bool,
}

/// A floating "+12" / "-60" on the RM note.
#[derive(Component)]
struct Transaction(Timer);

#[derive(Component)]
struct RmNote;

#[derive(Component)]
struct WaveNote;

#[derive(Component)]
struct BoatNote;

/// Overlay text that follows `subject`; despawned with it.
#[derive(Component)]
pub struct OverlayLabel {
    subject: Entity,
    offset: Vec2,
}

const PAPER: Color = Color::srgb(0.93, 0.88, 0.72);
const INK: Color = Color::srgb(0.16, 0.13, 0.10);

fn paper(size: f32) -> impl Bundle {
    (
        Text::new(String::new()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(INK),
        BackgroundColor(PAPER),
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            ..default()
        },
    )
}

fn spawn_notes(mut commands: Commands) {
    let mut rm = commands.spawn((
        Name::new("RM note"),
        DespawnOnExit(Screen::Gameplay),
        RmNote,
        paper(22.0),
    ));
    rm.entry::<Node>().and_modify(|mut node| {
        node.top = Val::Px(10.0);
        node.left = Val::Px(12.0);
    });

    let mut wave = commands.spawn((
        Name::new("Wave note"),
        DespawnOnExit(Screen::Gameplay),
        WaveNote,
        paper(15.0),
    ));
    wave.entry::<Node>().and_modify(|mut node| {
        node.top = Val::Px(10.0);
        node.right = Val::Px(12.0);
    });

    let mut boat = commands.spawn((
        Name::new("Boat note"),
        DespawnOnExit(Screen::Gameplay),
        BoatNote,
        paper(15.0),
    ));
    boat.entry::<Node>().and_modify(|mut node| {
        node.bottom = Val::Px(10.0);
        node.left = Val::Px(12.0);
    });
}

fn reset_marks(mut displayed: ResMut<DisplayedMarks>, treasury: Res<Treasury>) {
    displayed.shown = treasury.marks as f32;
    displayed.last = treasury.marks;
}

fn toggle_overlay(mut intents: MessageReader<ToggleOverlay>, mut overlay: ResMut<Overlay>) {
    for _ in intents.read() {
        overlay.on = !overlay.on;
    }
}

fn tween_marks(
    time: Res<Time<Real>>,
    treasury: Res<Treasury>,
    mut displayed: ResMut<DisplayedMarks>,
    mut commands: Commands,
    notes: Query<Entity, With<RmNote>>,
) {
    if treasury.marks != displayed.last {
        let delta = treasury.marks as i64 - displayed.last as i64;
        displayed.last = treasury.marks;
        for note in &notes {
            commands.spawn((
                Name::new("Transaction"),
                Transaction(Timer::from_seconds(1.2, TimerMode::Once)),
                DespawnOnExit(Screen::Gameplay),
                Text::new(format!("{delta:+}")),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(if delta > 0 {
                    Color::srgb(0.20, 0.45, 0.15)
                } else {
                    Color::srgb(0.65, 0.15, 0.10)
                }),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(100.0),
                    top: Val::Px(6.0),
                    margin: UiRect::left(Val::Px(8.0)),
                    ..default()
                },
                ChildOf(note),
            ));
        }
    }
    let target = treasury.marks as f32;
    let rate = (time.delta_secs() / RM_TWEEN_SECONDS).min(1.0);
    displayed.shown += (target - displayed.shown) * rate;
    if (displayed.shown - target).abs() < 0.5 {
        displayed.shown = target;
    }
}

fn float_transactions(
    time: Res<Time<Real>>,
    mut commands: Commands,
    mut floating: Query<(Entity, &mut Transaction, &mut Node, &mut TextColor)>,
) {
    for (entity, mut transaction, mut node, mut color) in &mut floating {
        if transaction.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = transaction.0.fraction();
        node.top = Val::Px(6.0 - 26.0 * progress);
        color.0.set_alpha(1.0 - progress);
    }
}

fn update_rm_note(mut note: Single<&mut Text, With<RmNote>>, displayed: Res<DisplayedMarks>) {
    note.0 = format!("RM {}", displayed.shown.round() as u32);
}

fn update_wave_note(
    mut note: Single<&mut Text, With<WaveNote>>,
    integrity: Res<Integrity>,
    wave: Res<WaveState>,
    time: Res<Time<Virtual>>,
    overlay: Res<Overlay>,
) {
    let phase = if wave.prologue {
        "Versenke den Goldtransport".to_string()
    } else if wave.running {
        String::new()
    } else {
        format!("[Enter] naechste: {}", wave.preview())
    };
    let pace = if time.is_paused() {
        "PAUSE [Esc]"
    } else if time.relative_speed() > 1.0 {
        "2x [F]"
    } else {
        "[F] 2x"
    };
    let tab = if overlay.on {
        "[Tab] Karte aus"
    } else {
        "[Tab] Karte"
    };

    note.0 = format!(
        "Welle {}\nBunker {}\n{}\n{}   {}",
        wave.label(),
        integrity.0,
        phase,
        pace,
        tab
    );
}

fn update_boat_note(
    mut note: Single<&mut Text, With<BoatNote>>,
    flotilla: Res<Flotilla>,
    boats: Query<(&Health, &Torpedoes, &Trim), With<UBoat>>,
) {
    let stats = flotilla.model.stats();
    let Ok((health, torpedoes, trim)) = boats.single() else {
        note.0 = format!("{} gesunken\nkehrt nach der Welle zurueck", stats.name);
        return;
    };

    let pips: String = (0..stats.tubes)
        .map(|i| if i < torpedoes.loaded { '|' } else { '.' })
        .collect();

    note.0 = format!(
        "{}   {}\nTorpedos [{}]   Rumpf {:.0}%\n[Rechtsklick] tauchen   [Space] Torpedo",
        stats.name,
        trim.label(),
        pips,
        health.fraction() * 100.0,
    );
}

fn draw_health_bars(mut gizmos: Gizmos, boats: Query<(&Transform, &Health), With<Enemy>>) {
    for (transform, health) in &boats {
        let fraction = health.fraction();
        if fraction >= 1.0 {
            continue;
        }
        let base = transform.translation.truncate() + Vec2::new(-14.0, -20.0);
        gizmos.line_2d(base, base + Vec2::X * 28.0, Color::srgb(0.25, 0.08, 0.08));
        gizmos.line_2d(
            base,
            base + Vec2::X * 28.0 * fraction,
            Color::srgb(0.90, 0.30, 0.25),
        );
    }
}

/// The enemy route: always faintly in the prep phase, otherwise only in the overlay.
fn draw_route(
    mut gizmos: Gizmos,
    path: Res<RiverPath>,
    wave: Res<WaveState>,
    overlay: Res<Overlay>,
) {
    if !(overlay.on || (!wave.running && !wave.prologue)) {
        return;
    }
    let ink = Color::srgba(0.85, 0.85, 0.7, 0.35);
    let mut s = 0.0;
    let dash = 18.0;
    while s < path.length() {
        gizmos.line_2d(
            path.point_at(s),
            path.point_at((s + dash * 0.6).min(path.length())),
            ink,
        );
        s += dash;
    }
}

fn draw_overlay(
    mut gizmos: Gizmos,
    overlay: Res<Overlay>,
    path: Res<RiverPath>,
    towers: Query<(&Transform, &Range, &Gun), With<Tower>>,
    shellers: Query<(&Transform, &Shelling), With<Enemy>>,
    boats: Query<(&Transform, &Station, &Heading), With<UBoat>>,
) {
    if !overlay.on {
        return;
    }
    let ink = Color::srgba(0.85, 0.85, 0.7, 0.5);
    for (transform, range, gun) in &towers {
        let at = transform.translation.truncate();
        gizmos.circle_2d(Isometry2d::from_translation(at), range.0, ink);
        gizmos.arc_2d(
            Isometry2d::from_translation(at),
            std::f32::consts::TAU * gun.reload.fraction(),
            20.0,
            Color::srgb(1.0, 0.95, 0.65),
        );
    }
    for (transform, shelling) in &shellers {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            shelling.range,
            Color::srgba(0.9, 0.3, 0.3, 0.5),
        );
    }
    for (transform, station, heading) in &boats {
        let tangent = path.tangent_at(station.along);
        let axis = if heading.0.dot(tangent) >= 0.0 {
            tangent
        } else {
            -tangent
        };
        let from = transform.translation.truncate();
        let end = path.straight_end(station.along, axis.dot(tangent) > 0.0);
        let to = path.point_at(end);
        let run = from.distance(to).min(TORPEDO_RUN);
        gizmos.line_2d(from, from + axis * run, Color::srgb(0.55, 0.90, 1.0));
    }
}

/// Every enemy, turret and the boat gets one overlay label when it appears.
fn attach_labels(
    mut commands: Commands,
    enemies: Query<Entity, Added<Enemy>>,
    towers: Query<Entity, Added<Tower>>,
    boats: Query<Entity, Added<UBoat>>,
) {
    let spawn = |commands: &mut Commands, subject: Entity, offset: Vec2| {
        commands.spawn((
            Name::new("Overlay label"),
            DespawnOnExit(Screen::Gameplay),
            OverlayLabel { subject, offset },
            Text2d::new(String::new()),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.95, 0.85)),
            Visibility::Hidden,
            Transform::from_translation(Vec3::new(0.0, 0.0, Z_UI)),
        ));
    };
    for enemy in &enemies {
        spawn(&mut commands, enemy, Vec2::new(0.0, 24.0));
    }
    for tower in &towers {
        spawn(&mut commands, tower, Vec2::new(0.0, -26.0));
    }
    for boat in &boats {
        spawn(&mut commands, boat, Vec2::new(0.0, -28.0));
    }
}

fn update_labels(
    overlay: Res<Overlay>,
    mut commands: Commands,
    mut labels: Query<(
        Entity,
        &OverlayLabel,
        &mut Text2d,
        &mut Visibility,
        &mut Transform,
    )>,
    subjects: Query<&Transform, Without<OverlayLabel>>,
    enemies: Query<&Health, With<Enemy>>,
    towers: Query<&Targeting, With<Tower>>,
    boats: Query<&Torpedoes, With<UBoat>>,
) {
    for (entity, label, mut text, mut visibility, mut transform) in &mut labels {
        let Ok(subject) = subjects.get(label.subject) else {
            commands.entity(entity).despawn();
            continue;
        };
        *visibility = if overlay.on {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if !overlay.on {
            continue;
        }
        transform.translation = (subject.translation.truncate() + label.offset).extend(Z_UI);
        text.0 = if let Ok(health) = enemies.get(label.subject) {
            format!("{:.0}/{:.0}", health.current, health.max)
        } else if let Ok(targeting) = towers.get(label.subject) {
            targeting.label().to_string()
        } else if let Ok(torpedoes) = boats.get(label.subject) {
            format!("{} Torpedos", torpedoes.loaded)
        } else {
            String::new()
        };
    }
}
