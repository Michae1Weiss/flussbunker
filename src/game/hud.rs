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
use crate::game::palette::{FAINT, GAIN, INK, NOTE, PENCIL, RED};
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
                scale_ui,
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

/// The paper behind the RM figure; transactions float from its edge.
#[derive(Component)]
struct RmPaper;

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

/// Logical window height the UI sizes are designed for.
const UI_REFERENCE_HEIGHT: f32 = 800.0;

/// A paper note pinned at `place`, its text in a child carrying `marker`.
fn spawn_note(
    commands: &mut Commands,
    name: &'static str,
    place: Node,
    size: f32,
    marker: impl Bundle,
) -> Entity {
    let paper = commands
        .spawn((
            Name::new(name),
            DespawnOnExit(Screen::Gameplay),
            BackgroundColor(NOTE),
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                ..place
            },
        ))
        .id();
    commands.spawn((
        marker,
        Text::new(String::new()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(INK),
        ChildOf(paper),
    ));
    paper
}

fn spawn_notes(mut commands: Commands) {
    let top_left = Node {
        top: Val::Px(10.0),
        left: Val::Px(12.0),
        ..default()
    };
    let rm = spawn_note(&mut commands, "RM note", top_left, 26.0, RmNote);
    commands.entity(rm).insert(RmPaper);
    let top_right = Node {
        top: Val::Px(10.0),
        right: Val::Px(12.0),
        ..default()
    };
    spawn_note(&mut commands, "Wave note", top_right, 19.0, WaveNote);
    let bottom_left = Node {
        bottom: Val::Px(10.0),
        left: Val::Px(12.0),
        ..default()
    };
    spawn_note(&mut commands, "Boat note", bottom_left, 19.0, BoatNote);
}

/// Sizes are designed at 800 logical pixels high; larger windows scale the UI up.
fn scale_ui(window: Single<&Window, Changed<Window>>, mut scale: ResMut<UiScale>) {
    let wanted = (window.height() / UI_REFERENCE_HEIGHT).clamp(1.0, 2.0);
    if (scale.0 - wanted).abs() > 0.01 {
        scale.0 = wanted;
    }
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
    notes: Query<Entity, With<RmPaper>>,
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
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(if delta > 0 { GAIN } else { RED }),
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
        format!("[Enter] nächste: {}", wave.preview())
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
        note.0 = format!("{} gesunken\nkehrt nach der Welle zurück", stats.name);
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
        gizmos.line_2d(base, base + Vec2::X * 28.0, FAINT);
        gizmos.line_2d(base, base + Vec2::X * 28.0 * fraction, RED);
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
    let ink = FAINT;
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
    let ink = PENCIL;
    for (transform, range, gun) in &towers {
        let at = transform.translation.truncate();
        gizmos.circle_2d(Isometry2d::from_translation(at), range.0, ink);
        gizmos.arc_2d(
            Isometry2d::from_translation(at),
            std::f32::consts::TAU * gun.reload.fraction(),
            20.0,
            INK,
        );
    }
    for (transform, shelling) in &shellers {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            shelling.range,
            RED.with_alpha(0.5),
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
        gizmos.line_2d(from, from + axis * run, RED);
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
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(INK),
            TextBackgroundColor(NOTE),
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
