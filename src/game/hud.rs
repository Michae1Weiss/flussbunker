//! The HUD: one status line, plus a health pip over every damaged boat.
//!
//! Deliberately ugly and deliberately cheap. Information clarity is where most
//! amateur tower defenses fail, and it is worth far more than a styled panel:
//! the player must be able to answer "why did that boat survive" without asking
//! the developer.

use bevy::prelude::*;

use crate::game::bunker::Integrity;
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::orders::SelectedTool;
use crate::game::uboat::UBoatPen;
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud)
        .add_systems(
            Update,
            (update_status, draw_health_bars).in_set(Frame::Present),
        );
}

#[derive(Component)]
struct StatusLine;

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("HUD"),
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(12.0),
            ..default()
        },
        children![(
            StatusLine,
            Text::new(String::new()),
            TextFont {
                font_size: FontSize::Px(17.0),
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.90, 0.86)),
        )],
    ));
}

/// Rewrites the whole line every frame. At one text node that is free; if this
/// were forty labels the fix would be change detection (`Changed<Treasury>`),
/// not a template engine.
fn update_status(
    mut status: Single<&mut Text, With<StatusLine>>,
    treasury: Res<Treasury>,
    integrity: Res<Integrity>,
    wave: Res<WaveState>,
    pen: Res<UBoatPen>,
    tool: Res<SelectedTool>,
) {
    let building = match &pen.building {
        Some(timer) => format!(" (baut {:.0}s)", timer.remaining_secs()),
        None => String::new(),
    };
    let phase = if wave.running {
        "Welle laeuft"
    } else {
        "SPACE: naechste Welle"
    };

    status.0 = format!(
        "RM {}   Bunker {}   Welle {}   U-Boote {} bereit / {} in See{}   Werkzeug: {:?}   {}",
        treasury.marks,
        integrity.0,
        wave.label(),
        pen.docked,
        pen.at_sea,
        building,
        *tool,
        phase,
    );
}

/// A red pip under every damaged boat. Gizmos again: immediate mode means no
/// entities to spawn, follow or clean up, which is exactly right for something
/// that exists for one frame.
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
