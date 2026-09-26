//! The main menu over the painted map: play, settings, quit.

use bevy::prelude::*;

use super::widget::{button, card, heading, veil, Size};
use super::{Menu, Screen};
use crate::game::art::Art;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), open)
        .add_systems(OnEnter(Menu::Main), spawn);
}

fn open(mut menu: ResMut<NextState<Menu>>) {
    menu.set(Menu::Main);
}

fn spawn(mut commands: Commands, art: Res<Art>) {
    let root = veil(&mut commands, "Main menu", DespawnOnExit(Menu::Main));
    heading(&mut commands, root, "Flussbunker", 72.0);
    let card = card(&mut commands, &art, root);
    button(&mut commands, &art, card, "Spielen", Size::Big, play);
    button(
        &mut commands,
        &art,
        card,
        "Einstellungen",
        Size::Normal,
        |_: On<Pointer<Click>>, mut menu: ResMut<NextState<Menu>>| menu.set(Menu::Settings),
    );
    #[cfg(not(target_arch = "wasm32"))]
    button(
        &mut commands,
        &art,
        card,
        "Beenden",
        Size::Normal,
        |_: On<Pointer<Click>>, mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        },
    );
}

fn play(
    _: On<Pointer<Click>>,
    mut screen: ResMut<NextState<Screen>>,
    mut menu: ResMut<NextState<Menu>>,
) {
    screen.set(Screen::Gameplay);
    menu.set(Menu::None);
}
