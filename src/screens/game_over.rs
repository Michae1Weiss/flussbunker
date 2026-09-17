//! Victory and defeat screens. One plugin: they differ by a string.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use super::{line, overlay_root, Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Victory), spawn_victory)
        .add_systems(OnEnter(Screen::Defeat), spawn_defeat)
        .add_systems(
            Update,
            back_to_title.run_if(
                (in_state(Screen::Victory).or_else(in_state(Screen::Defeat)))
                    .and_then(input_just_pressed(KeyCode::Space)),
            ),
        );
}

fn spawn_victory(mut commands: Commands) {
    commands.spawn((
        Name::new("Victory UI"),
        DespawnOnExit(Screen::Victory),
        overlay_root(),
        children![
            line("DER FLUSS HAELT", 48.0, [0.70, 0.90, 0.72]),
            line("Alle Wellen abgewehrt.", 22.0, [0.72, 0.76, 0.80]),
            line("[ SPACE ]", 26.0, [0.92, 0.88, 0.60]),
        ],
    ));
}

fn spawn_defeat(mut commands: Commands) {
    commands.spawn((
        Name::new("Defeat UI"),
        DespawnOnExit(Screen::Defeat),
        overlay_root(),
        children![
            line("BUNKER DURCHBROCHEN", 48.0, [0.90, 0.55, 0.50]),
            line("Die Boote sind durchgekommen.", 22.0, [0.72, 0.76, 0.80]),
            line("[ SPACE ]", 26.0, [0.92, 0.88, 0.60]),
        ],
    ));
}

fn back_to_title(mut next: ResMut<NextState<Screen>>) {
    next.set(Screen::Title);
}
