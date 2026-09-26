//! Esc in the game: the clock stops under a paper menu. Resume, restart, settings, main menu.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use super::widget::{button, card, heading, veil, Size};
use super::{Menu, Screen};
use crate::game::art::Art;
use crate::game::input::TogglePause;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), (stop_clock, spawn))
        .add_systems(OnEnter(Menu::None), run_clock)
        .add_systems(OnExit(Screen::Gameplay), resume)
        .add_systems(
            Update,
            (
                open.run_if(in_state(Screen::Gameplay).and_then(in_state(Menu::None))),
                resume.run_if(in_state(Menu::Pause).and_then(input_just_pressed(KeyCode::Escape))),
            ),
        );
}

fn open(mut intents: MessageReader<TogglePause>, mut menu: ResMut<NextState<Menu>>) {
    if intents.read().count() > 0 {
        menu.set(Menu::Pause);
    }
}

fn resume(mut menu: ResMut<NextState<Menu>>) {
    menu.set(Menu::None);
}

fn stop_clock(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn run_clock(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

fn spawn(mut commands: Commands, art: Res<Art>) {
    let root = veil(&mut commands, "Pause menu", DespawnOnExit(Menu::Pause));
    let card = card(&mut commands, &art, root);
    heading(&mut commands, card, "Pause", 52.0);
    button(
        &mut commands,
        &art,
        card,
        "Weiter",
        Size::Normal,
        |_: On<Pointer<Click>>, mut menu: ResMut<NextState<Menu>>| menu.set(Menu::None),
    );
    button(
        &mut commands,
        &art,
        card,
        "Neustart",
        Size::Normal,
        |_: On<Pointer<Click>>,
         mut screen: ResMut<NextState<Screen>>,
         mut menu: ResMut<NextState<Menu>>| {
            screen.set(Screen::Gameplay);
            menu.set(Menu::None);
        },
    );
    button(
        &mut commands,
        &art,
        card,
        "Einstellungen",
        Size::Normal,
        |_: On<Pointer<Click>>, mut menu: ResMut<NextState<Menu>>| menu.set(Menu::Settings),
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
