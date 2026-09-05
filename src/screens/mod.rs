//! Screen flow. One state enum, one plugin per screen.
//!
//! Small games can absolutely be flat (no states at all) — that is a real,
//! defensible choice. We take states here for one concrete reason: cleanup.
//! `DespawnOnExit(Screen::X)` deletes every entity tagged with it on exit, so
//! restarting a run needs no teardown code and cannot leak entities.

use bevy::prelude::*;

mod game_over;
mod title;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Title,
    Gameplay,
    Victory,
    Defeat,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .add_plugins((title::plugin, game_over::plugin));
}

/// Shared full-screen centred column. A tiny `impl Bundle` helper instead of a
/// widget framework: two screens is not enough repetition to justify one.
pub fn overlay_root() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(14.0),
            ..default()
        },
        // Nothing draws behind the title screen, so a background colour here
        // doubles as the screen itself.
        BackgroundColor(Color::srgba(0.03, 0.05, 0.07, 0.88)),
    )
}

pub fn line(text: &str, size: f32, rgb: [f32; 3]) -> impl Bundle {
    (
        Text::new(text.to_string()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(rgb[0], rgb[1], rgb[2])),
    )
}
