//! The HUD, rebuilt around one playtest finding: *three rounds were played
//! without discovering a core mechanic.*
//!
//! That is not a player mistake. If a rule is not on screen, it does not exist.
//! So this file now carries:
//!
//! * a **build bar** with every tower's name, hotkey and cost, greyed out when
//!   unaffordable — "u-boat cost and the limit of two is not clear" was a HUD
//!   bug, not a design one;
//! * a **boat panel**: rank, torpedoes as pips, trim state, hull;
//! * a **per-wave hint** during prep, because a player who has never played a
//!   tower defense has no conventions to fall back on and should not have to.
//!
//! Still deliberately plain. Information clarity outranks styling by a wide
//! margin, and every minute spent on a panel border is a minute not spent
//! making the wave curve fair.

use bevy::prelude::*;

use crate::balance::{rank_for, TowerKind, UBOAT_REPLACEMENT_COST, WAVES};
use crate::game::bunker::Integrity;
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::Enemy;
use crate::game::input::SelectedTool;
use crate::game::uboat::{Flotilla, Torpedoes, Trim, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_hud)
        .add_systems(
            Update,
            (
                update_status,
                update_build_bar,
                update_boat_panel,
                draw_health_bars,
            )
                .in_set(Frame::Present),
        );
}

#[derive(Component)]
struct StatusLine;

#[derive(Component)]
struct BoatPanel;

/// Marks one build-bar entry, so the update system knows which tower each text
/// node is describing. A marker component carrying an index is the ECS answer
/// to "which widget is this" — no lookup table, no name matching.
#[derive(Component)]
struct BuildSlot(usize);

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

    let mut bar = commands.spawn((
        Name::new("Build bar"),
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(18.0),
            ..default()
        },
    ));
    bar.with_children(|parent| {
        for index in 0..TowerKind::ALL.len() {
            parent.spawn((BuildSlot(index), text(16.0)));
        }
    });
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
            "   [SPACE] Welle {} starten: {}",
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

/// Greying out unaffordable options is the whole feature. A price the player
/// cannot see is a rule the player cannot learn.
fn update_build_bar(
    treasury: Res<Treasury>,
    tool: Res<SelectedTool>,
    mut slots: Query<(&BuildSlot, &mut Text, &mut TextColor)>,
) {
    for (slot, mut text, mut color) in &mut slots {
        let kind = TowerKind::ALL[slot.0];
        let stats = kind.stats();
        let selected = tool.0 == slot.0;
        let affordable = treasury.marks >= stats.cost;

        text.0 = format!(
            "{}[{}] {}  {} RM",
            if selected { "> " } else { "  " },
            stats.hotkey,
            stats.name,
            stats.cost
        );
        color.0 = match (selected, affordable) {
            (true, true) => Color::srgb(1.0, 0.95, 0.65),
            (false, true) => Color::srgb(0.80, 0.84, 0.80),
            (_, false) => Color::srgb(0.42, 0.40, 0.38),
        };
    }
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

    // Torpedoes as pips rather than a number: four filled bars are read at a
    // glance, "4/6" is read by parsing. Glanceable beats precise on a HUD.
    let pips: String = (0..rank.torpedoes)
        .map(|i| if i < torpedoes.loaded { '|' } else { '.' })
        .collect();

    panel.0 = format!(
        "U-96  {} ({} Abschuesse)   Torpedos [{}]   Rumpf {:.0}%   {}   [T] tauchen",
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
