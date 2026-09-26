//! Settings, from the main menu or the pause menu: music and sound volume, hints on or off.
//! Back (or Esc) returns to the menu it came from. Sole writer of `Settings`.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

use super::widget::{button, card, heading, veil, Label, Size};
use super::{Menu, Screen};
use crate::game::art::Art;
use crate::game::palette::{FAINT, INK};
use crate::settings::Settings;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn).add_systems(
        Update,
        (
            show.run_if(in_state(Menu::Settings)),
            back_on_escape
                .run_if(in_state(Menu::Settings).and_then(input_just_pressed(KeyCode::Escape))),
        ),
    );
}

#[derive(Clone, Copy, PartialEq)]
enum Channel {
    Music,
    Sound,
}

fn level(settings: &Settings, channel: Channel) -> u8 {
    match channel {
        Channel::Music => settings.music,
        Channel::Sound => settings.sound,
    }
}

fn level_mut(settings: &mut Settings, channel: Channel) -> &mut u8 {
    match channel {
        Channel::Music => &mut settings.music,
        Channel::Sound => &mut settings.sound,
    }
}

/// One mark of a volume bar, inked while the volume reaches it.
#[derive(Component)]
struct Mark {
    channel: Channel,
    index: u8,
}

#[derive(Component)]
struct HintsSwitch;

fn spawn(mut commands: Commands, art: Res<Art>) {
    let root = veil(
        &mut commands,
        "Settings menu",
        DespawnOnExit(Menu::Settings),
    );
    let card = card(&mut commands, &art, root);
    heading(&mut commands, card, "Einstellungen", 44.0);
    volume_row(&mut commands, &art, card, "Musik", Channel::Music);
    volume_row(&mut commands, &art, card, "Sound", Channel::Sound);
    let row = row(&mut commands, card, "Hinweise");
    let switch = button(
        &mut commands,
        &art,
        row,
        "",
        Size::Small,
        |_: On<Pointer<Click>>, mut settings: ResMut<Settings>| settings.hints = !settings.hints,
    );
    commands.entity(switch).insert(HintsSwitch);
    button(&mut commands, &art, card, "Zurück", Size::Normal, back);
}

/// A labelled row; the label keeps the controls of all rows aligned.
fn row(commands: &mut Commands, parent: Entity, label: &str) -> Entity {
    let row = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    commands.spawn((
        Text::new(label),
        TextFont {
            font_size: FontSize::Px(30.0),
            ..default()
        },
        TextColor(INK),
        Node {
            width: Val::Px(130.0),
            ..default()
        },
        ChildOf(row),
    ));
    row
}

fn volume_row(commands: &mut Commands, art: &Art, parent: Entity, label: &str, channel: Channel) {
    let row = row(commands, parent, label);
    button(
        commands,
        art,
        row,
        "-",
        Size::Small,
        move |_: On<Pointer<Click>>, mut settings: ResMut<Settings>| {
            let level = level_mut(&mut settings, channel);
            *level = level.saturating_sub(1);
        },
    );
    for index in 0..Settings::STEPS {
        commands.spawn((
            Mark { channel, index },
            Node {
                width: Val::Px(10.0),
                height: Val::Px(26.0),
                ..default()
            },
            BackgroundColor(FAINT),
            ChildOf(row),
        ));
    }
    button(
        commands,
        art,
        row,
        "+",
        Size::Small,
        move |_: On<Pointer<Click>>, mut settings: ResMut<Settings>| {
            let level = level_mut(&mut settings, channel);
            *level = (*level + 1).min(Settings::STEPS);
        },
    );
}

fn show(
    settings: Res<Settings>,
    mut marks: Query<(&Mark, &mut BackgroundColor)>,
    switches: Query<&Children, With<HintsSwitch>>,
    mut labels: Query<&mut Text, With<Label>>,
) {
    for (mark, mut color) in &mut marks {
        let inked = mark.index < level(&settings, mark.channel);
        color.0 = if inked { INK } else { FAINT };
    }
    let wanted = if settings.hints { "An" } else { "Aus" };
    for children in &switches {
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                if text.0 != wanted {
                    text.0 = wanted.into();
                }
            }
        }
    }
}

fn from(screen: &Screen) -> Menu {
    if *screen == Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    }
}

fn back(_: On<Pointer<Click>>, screen: Res<State<Screen>>, mut menu: ResMut<NextState<Menu>>) {
    menu.set(from(screen.get()));
}

fn back_on_escape(screen: Res<State<Screen>>, mut menu: ResMut<NextState<Menu>>) {
    menu.set(from(screen.get()));
}
