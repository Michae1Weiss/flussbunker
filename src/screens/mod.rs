//! Screen flow: one `Screen` state; `DespawnOnExit` does all cleanup.

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

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .add_plugins((title::plugin, game_over::plugin));
}

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
