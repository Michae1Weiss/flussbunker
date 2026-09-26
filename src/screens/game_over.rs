//! Victory and defeat: one card over the map, again or back to the main menu.

use bevy::prelude::*;

use super::widget::{button, card, heading, veil, Size};
use super::Screen;
use crate::game::art::Art;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Victory), |c: Commands, a: Res<Art>| {
        spawn(c, a, "Der Fluss hält", Screen::Victory)
    })
    .add_systems(OnEnter(Screen::Defeat), |c: Commands, a: Res<Art>| {
        spawn(c, a, "Bunker durchbrochen", Screen::Defeat)
    });
}

fn spawn(mut commands: Commands, art: Res<Art>, title: &str, screen: Screen) {
    let root = veil(&mut commands, "End card", DespawnOnExit(screen));
    let card = card(&mut commands, &art, root);
    heading(&mut commands, card, title, 52.0);
    button(
        &mut commands,
        &art,
        card,
        "Nochmal",
        Size::Big,
        |_: On<Pointer<Click>>, mut screen: ResMut<NextState<Screen>>| screen.set(Screen::Gameplay),
    );
    button(
        &mut commands,
        &art,
        card,
        "Hauptmenü",
        Size::Normal,
        |_: On<Pointer<Click>>, mut screen: ResMut<NextState<Screen>>| screen.set(Screen::Title),
    );
}
