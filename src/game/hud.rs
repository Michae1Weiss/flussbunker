//! Status line with tweened RM and floating transactions, boat panel, enemy health
//! bars. Reads other features' state; sole writer of `DisplayedMarks`.

use bevy::prelude::*;

use crate::balance::RM_TWEEN_SECONDS;
use crate::game::bunker::Integrity;
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::uboat::{Flotilla, Torpedoes, Trim, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<DisplayedMarks>()
        .add_systems(OnEnter(Screen::Gameplay), (spawn_hud, reset_marks))
        .add_systems(
            Update,
            (
                tween_marks,
                update_status,
                update_boat_panel,
                float_transactions,
                draw_health_bars,
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

/// A floating "+12" / "-60" next to the status line.
#[derive(Component)]
struct Transaction(Timer);

#[derive(Component)]
struct StatusLine;

#[derive(Component)]
struct BoatPanel;

fn text(size: f32) -> impl Bundle {
    (
        Text::new(String::new()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.88, 0.90, 0.86)),
    )
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("HUD top"),
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        children![(StatusLine, text(17.0)), (BoatPanel, text(15.0))],
    ));
}

fn reset_marks(mut displayed: ResMut<DisplayedMarks>, treasury: Res<Treasury>) {
    displayed.shown = treasury.marks as f32;
    displayed.last = treasury.marks;
}

fn tween_marks(
    time: Res<Time<Real>>,
    treasury: Res<Treasury>,
    mut displayed: ResMut<DisplayedMarks>,
    mut commands: Commands,
    lines: Query<Entity, With<StatusLine>>,
) {
    if treasury.marks != displayed.last {
        let delta = treasury.marks as i64 - displayed.last as i64;
        displayed.last = treasury.marks;
        for line in &lines {
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
                    Color::srgb(0.7, 0.95, 0.6)
                } else {
                    Color::srgb(0.95, 0.6, 0.5)
                }),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(22.0),
                    ..default()
                },
                ChildOf(line),
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
        node.top = Val::Px(22.0 + 26.0 * progress);
        color.0.set_alpha(1.0 - progress);
    }
}

fn update_status(
    mut status: Single<&mut Text, With<StatusLine>>,
    displayed: Res<DisplayedMarks>,
    integrity: Res<Integrity>,
    wave: Res<WaveState>,
    time: Res<Time<Virtual>>,
) {
    let phase = if wave.prologue {
        "   Versenke den Goldtransport: Linksklick faehrt, Space feuert".to_string()
    } else if wave.running {
        String::new()
    } else {
        format!("   [ENTER] naechste Welle: {}", wave.preview())
    };
    let pace = if time.relative_speed() > 1.0 {
        "   2x [F]"
    } else {
        ""
    };

    status.0 = format!(
        "RM {}    Bunker {}    Welle {}{}{}",
        displayed.shown.round() as u32,
        integrity.0,
        wave.label(),
        phase,
        pace,
    );
}

fn update_boat_panel(
    mut panel: Single<&mut Text, With<BoatPanel>>,
    flotilla: Res<Flotilla>,
    boats: Query<(&Health, &Torpedoes, &Trim), With<UBoat>>,
) {
    let stats = flotilla.model.stats();
    let Ok((health, torpedoes, trim)) = boats.single() else {
        panel.0 = format!("{} gesunken - kehrt nach der Welle zurueck", stats.name);
        return;
    };

    let pips: String = (0..stats.tubes)
        .map(|i| if i < torpedoes.loaded { '|' } else { '.' })
        .collect();

    panel.0 = format!(
        "{}   Torpedos [{}]   Rumpf {:.0}%   {}   [Rechtsklick] tauchen  [Space] Torpedo",
        stats.name,
        pips,
        health.fraction() * 100.0,
        trim.label(),
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
