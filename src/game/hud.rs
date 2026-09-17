//! Status line, boat panel, enemy health bars. Reads other features'
//! state, writes none of it.

use bevy::prelude::*;

use crate::balance::{rank_for, UBOAT_REPLACEMENT_COST, WAVES};
use crate::game::bunker::Integrity;
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::uboat::{Flotilla, Torpedoes, Trim, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud)
        .add_systems(
            Update,
            (update_status, update_boat_panel, draw_health_bars).in_set(Frame::Present),
        );
}

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

fn update_status(
    mut status: Single<&mut Text, With<StatusLine>>,
    treasury: Res<Treasury>,
    integrity: Res<Integrity>,
    wave: Res<WaveState>,
) {
    let phase = if wave.running {
        String::new()
    } else if wave.index < WAVES.len() {
        format!(
            "   [ENTER] Welle {} starten: {}",
            wave.index + 1,
            WAVES[wave.index].hint
        )
    } else {
        String::new()
    };

    status.0 = format!(
        "RM {}    Bunker {}    Welle {}{}",
        treasury.marks,
        integrity.0,
        wave.label(),
        phase,
    );
}

fn update_boat_panel(
    mut panel: Single<&mut Text, With<BoatPanel>>,
    flotilla: Res<Flotilla>,
    boats: Query<(&Health, &Torpedoes, &Trim), With<UBoat>>,
) {
    let rank = rank_for(flotilla.kills);

    let Ok((health, torpedoes, trim)) = boats.single() else {
        panel.0 = format!(
            "U-96 VERLOREN    [N] Neubau {} RM    (Rang und Abschuesse sind weg)",
            UBOAT_REPLACEMENT_COST
        );
        return;
    };

    let pips: String = (0..rank.torpedoes)
        .map(|i| if i < torpedoes.loaded { '|' } else { '.' })
        .collect();

    panel.0 = format!(
        "U-96  {} ({} Abschuesse)   Torpedos [{}]   Rumpf {:.0}%   {}   [Rechtsklick] tauchen  [Space] Torpedo",
        rank.name,
        flotilla.kills,
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
