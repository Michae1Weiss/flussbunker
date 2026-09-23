//! Big paper banners that announce a change of phase: prologue, wave start, last wave, wave
//! cleared, boat lost, pause. They never take input and never stop the game (pillar 3).
//! Sole writer of `Banner`. Rules: docs/design/ui-input.md

use bevy::prelude::*;

use crate::balance::{PROLOGUE_BOUNTY, WAVES, WAVE_CLEAR_BONUS};
use crate::game::art::{paper_backing, Art};
use crate::game::combat::Sunk;
use crate::game::palette::{INK, NOTE, PENCIL};
use crate::game::uboat::UBoat;
use crate::game::wave::{WaveFinished, WaveStarted, WaveState};
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), announce_prologue)
        .add_systems(
            Update,
            (announce_waves, show_pause, animate)
                .chain()
                .in_set(Frame::Present),
        )
        .add_observer(announce_boat_lost);
}

/// Seconds to slam in, to stay, and to slide out; real time, so the pause banner moves too.
const SLAM: f32 = 0.25;
const HOLD: f32 = 1.6;
const LEAVE: f32 = 0.3;

/// `age` in real seconds; a banner with `stays` waits until something removes it.
#[derive(Component)]
pub struct Banner {
    age: f32,
    stays: bool,
}

#[derive(Component)]
struct PauseBanner;

/// The paper that animates; a child of the full-width row that centres it.
#[derive(Component)]
struct Sheet;

fn spawn_banner(
    commands: &mut Commands,
    art: &Art,
    title: String,
    subtitle: String,
    stays: bool,
) -> Entity {
    let row = commands
        .spawn((
            Name::new("Banner"),
            Banner { age: 0.0, stays },
            DespawnOnExit(Screen::Gameplay),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(20.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .id();
    let backing = paper_backing(art, "ui/banner.png");
    let mut sheet = commands.spawn((
        Sheet,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(48.0), Val::Px(18.0)),
            ..default()
        },
        UiTransform {
            rotation: Rot2::degrees(-2.0),
            ..default()
        },
        ChildOf(row),
    ));
    if backing.is_none() {
        sheet.insert(BackgroundColor(NOTE));
    }
    let sheet = sheet.id();
    if let Some(backing) = backing {
        commands.spawn((backing, ChildOf(sheet)));
    }
    commands.spawn((
        Text::new(title),
        TextFont {
            font_size: FontSize::Px(46.0),
            ..default()
        },
        TextColor(INK),
        ChildOf(sheet),
    ));
    if !subtitle.is_empty() {
        commands.spawn((
            Text::new(subtitle),
            TextFont {
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(PENCIL),
            ChildOf(sheet),
        ));
    }
    row
}

/// A new announcement replaces the one showing; the pause banner is left alone.
fn clear(commands: &mut Commands, banners: &Query<(Entity, &Banner), Without<PauseBanner>>) {
    for (entity, _) in banners {
        commands.entity(entity).despawn();
    }
}

fn announce_prologue(mut commands: Commands, art: Res<Art>) {
    spawn_banner(
        &mut commands,
        &art,
        "Versenke den Goldtransport".into(),
        "Er darf den Bunker nicht erreichen".into(),
        false,
    );
}

fn announce_waves(
    mut commands: Commands,
    art: Res<Art>,
    wave: Res<WaveState>,
    mut started: MessageReader<WaveStarted>,
    mut finished: MessageReader<WaveFinished>,
    banners: Query<(Entity, &Banner), Without<PauseBanner>>,
) {
    for start in started.read() {
        clear(&mut commands, &banners);
        let last = start.index + 1 == WAVES.len();
        let title = if last {
            "Letzte Welle".to_string()
        } else {
            format!("Welle {}", start.index + 1)
        };
        spawn_banner(
            &mut commands,
            &art,
            title,
            WAVES[start.index].name.into(),
            false,
        );
    }
    if finished.read().count() == 0 || wave.index >= WAVES.len() {
        return;
    }
    clear(&mut commands, &banners);
    let (title, subtitle) = if wave.index == 0 {
        (
            "Goldtransport versenkt".to_string(),
            format!("+{PROLOGUE_BOUNTY} RM. Jetzt bauen, dann Enter"),
        )
    } else {
        (
            "Welle geschafft".to_string(),
            format!("+{WAVE_CLEAR_BONUS} RM. Enter: nächste Welle"),
        )
    };
    spawn_banner(&mut commands, &art, title, subtitle, false);
}

fn announce_boat_lost(
    sunk: On<Sunk>,
    mut commands: Commands,
    art: Res<Art>,
    boats: Query<(), With<UBoat>>,
    banners: Query<(Entity, &Banner), Without<PauseBanner>>,
) {
    if !boats.contains(sunk.entity) {
        return;
    }
    clear(&mut commands, &banners);
    spawn_banner(
        &mut commands,
        &art,
        "U-Boot verloren".into(),
        "Es kehrt nach der Welle zurück".into(),
        false,
    );
}

/// Only a pause the player chose: a stopped clock at speed 0 is not one.
fn show_pause(
    mut commands: Commands,
    art: Res<Art>,
    time: Res<Time<Virtual>>,
    pause: Query<Entity, With<PauseBanner>>,
) {
    match (time.is_paused(), pause.single()) {
        (true, Err(_)) => {
            let banner = spawn_banner(
                &mut commands,
                &art,
                "Pause".into(),
                "Esc: weiter".into(),
                true,
            );
            commands.entity(banner).insert(PauseBanner);
        }
        (false, Ok(entity)) => commands.entity(entity).despawn(),
        _ => {}
    }
}

/// Slams in (1.3 to 1.0), holds, slides up and away.
fn animate(
    time: Res<Time<Real>>,
    mut commands: Commands,
    mut banners: Query<(Entity, &mut Banner, &Children)>,
    mut sheets: Query<&mut UiTransform, With<Sheet>>,
) {
    for (entity, mut banner, children) in &mut banners {
        banner.age += time.delta_secs();
        let t = banner.age;
        if !banner.stays && t > SLAM + HOLD + LEAVE {
            commands.entity(entity).despawn();
            continue;
        }
        let slam = (t / SLAM).min(1.0);
        let scale = 1.3 - 0.3 * (1.0 - (1.0 - slam).powi(3));
        let leave = if banner.stays {
            0.0
        } else {
            ((t - SLAM - HOLD) / LEAVE).clamp(0.0, 1.0)
        };
        for &child in children {
            if let Ok(mut transform) = sheets.get_mut(child) {
                transform.scale = Vec2::splat(scale);
                transform.translation = Val2::px(0.0, -300.0 * leave * leave);
            }
        }
    }
}
