//! Title screen: rules card plus "press Space".
//!
//! Also the tutorial. A tower defense that explains armour in a tooltip has
//! already lost the player; explaining it before the first wave costs one
//! screen and prevents the "why is nothing dying" moment on wave 4.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use super::{line, overlay_root, Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title)
        .add_systems(
            Update,
            start_run.run_if(in_state(Screen::Title).and(input_just_pressed(KeyCode::Space))),
        );
}

fn spawn_title(mut commands: Commands) {
    commands.spawn((
        Name::new("Title UI"),
        // The tag that makes teardown disappear as a problem.
        DespawnOnExit(Screen::Title),
        overlay_root(),
        children![
            line("BUNKER VALENTIN", 52.0, [0.90, 0.86, 0.70]),
            line("Halte die Weser. U-96 gehoert dir - verliere sie nicht.", 20.0, [0.72, 0.76, 0.80]),
            line("1 / 2 / 3   Geschuetz waehlen  -  Linksklick auf Land: bauen", 18.0, [0.66, 0.70, 0.74]),
            line("Linksklick auf den Fluss: U-96 faehrt dorthin", 18.0, [0.66, 0.70, 0.74]),
            line("T  tauchen / auftauchen     N  neues U-Boot (nur wenn verloren)", 18.0, [0.66, 0.70, 0.74]),
            line("Space  naechste Welle starten (kein Auto-Start)", 18.0, [0.66, 0.70, 0.74]),
            line("Torpedos laufen GERADEAUS den Fluss entlang.", 19.0, [0.60, 0.88, 1.00]),
            line("Lange Geraden sind Schusslinien. Kurven sind totes Wasser.", 19.0, [0.60, 0.88, 1.00]),
            line("Zerstoerer sind gepanzert - und sie jagen dich. Roter Strahl: tauchen.", 19.0, [0.82, 0.55, 0.50]),
            line("[ SPACE ]", 30.0, [0.92, 0.88, 0.60]),
        ],
    ));
}

fn start_run(mut next: ResMut<NextState<Screen>>) {
    // Plain `set` is correct here because Title != Gameplay. Watch out for the
    // 0.19 change when a transition *might* be same-state: `set` fires the
    // exit/enter schedules regardless, which would despawn a live screen.
    next.set(Screen::Gameplay);
}
