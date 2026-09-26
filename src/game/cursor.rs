//! The mouse cursor says what a click would do: a pencil by default, a steering mark over the
//! water, a hammer over an empty build spot, a hand over anything that opens or presses.
//! Drawn cursors where they exist, system cursors otherwise. Sole writer of `CursorIcon`.

use bevy::prelude::*;
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage, PrimaryWindow, SystemCursorIcon};

use crate::game::art::Art;
use crate::game::input::Pointing;
use crate::screens::{Menu, Screen};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, set_cursor);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Look {
    Pencil,
    Steer,
    Build,
    Hand,
}

impl Look {
    /// The drawing, its hotspot in pixels, and the system cursor standing in for it.
    fn parts(self) -> (&'static str, (u16, u16), SystemCursorIcon) {
        match self {
            Look::Pencil => ("ui/cursors/pencil.png", (2, 2), SystemCursorIcon::Default),
            Look::Steer => (
                "ui/cursors/steer.png",
                (24, 24),
                SystemCursorIcon::Crosshair,
            ),
            Look::Build => ("ui/cursors/build.png", (4, 4), SystemCursorIcon::Pointer),
            Look::Hand => ("ui/cursors/hand.png", (8, 2), SystemCursorIcon::Pointer),
        }
    }
}

fn set_cursor(
    mut commands: Commands,
    window: Single<Entity, With<PrimaryWindow>>,
    art: Res<Art>,
    screen: Res<State<Screen>>,
    menu: Res<State<Menu>>,
    pointing: Res<Pointing>,
    buttons: Query<&Interaction, With<Button>>,
    mut current: Local<Option<Look>>,
) {
    let on_map = *screen.get() == Screen::Gameplay && *menu.get() == Menu::None;
    let look = if buttons.iter().any(|i| *i != Interaction::None) {
        Look::Hand
    } else if !on_map {
        Look::Pencil
    } else {
        match *pointing {
            Pointing::Nothing => Look::Pencil,
            Pointing::Water => Look::Steer,
            Pointing::Spot => Look::Build,
            Pointing::Turret | Pointing::Boat | Pointing::Entry => Look::Hand,
        }
    };
    if *current == Some(look) {
        return;
    }
    *current = Some(look);
    let (key, hotspot, system) = look.parts();
    let icon = match art.image(key) {
        Some(handle) => CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle,
            hotspot,
            ..default()
        })),
        None => CursorIcon::System(system),
    };
    commands.entity(*window).insert(icon);
}
