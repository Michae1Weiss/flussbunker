//! Big paper banners that announce a change of phase: prologue, wave start, last wave, wave
//! cleared, boat lost. They never take input and never stop the game (pillar 3).
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
            (announce_waves, animate).chain().in_set(Frame::Present),
        )
        .add_observer(announce_boat_lost);
}

/// Seconds to slam in, to stay, and to slide out.
const SLAM: f32 = 0.25;
const HOLD: f32 = 1.6;
const LEAVE: f32 = 0.3;

/// `age` in real seconds.
#[derive(Component)]
pub struct Banner {
    age: f32,
}

/// The paper that animates; a child of the full-width row that centres it.
#[derive(Component)]
struct Sheet;

fn spawn_banner(commands: &mut Commands, art: &Art, title: String, subtitle: String) {
    let row = commands
        .spawn((
            Name::new("Banner"),
            Banner { age: 0.0 },
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
}

/// A new announcement replaces the one showing.
fn clear(commands: &mut Commands, banners: &Query<Entity, With<Banner>>) {
    for entity in banners {
        commands.entity(entity).despawn();
    }
}

fn announce_prologue(mut commands: Commands, art: Res<Art>) {
    spawn_banner(
        &mut commands,
        &art,
        "Goldtransport".into(),
        "Versenken!".into(),
    );
}

fn announce_waves(
    mut commands: Commands,
    art: Res<Art>,
    wave: Res<WaveState>,
    mut started: MessageReader<WaveStarted>,
    mut finished: MessageReader<WaveFinished>,
    banners: Query<Entity, With<Banner>>,
) {
    for start in started.read() {
        clear(&mut commands, &banners);
        let last = start.index + 1 == WAVES.len();
        let title = if last {
            "Letzte Welle".to_string()
        } else {
            format!("Welle {}", start.index + 1)
        };
        spawn_banner(&mut commands, &art, title, WAVES[start.index].name.into());
    }
    if finished.read().count() == 0 || wave.index >= WAVES.len() {
        return;
    }
    clear(&mut commands, &banners);
    let (title, subtitle) = if wave.index == 0 {
        ("Versenkt!".to_string(), format!("+{PROLOGUE_BOUNTY} RM"))
    } else {
        (
            "Welle geschafft".to_string(),
            format!("+{WAVE_CLEAR_BONUS} RM"),
        )
    };
    spawn_banner(&mut commands, &art, title, subtitle);
}

fn announce_boat_lost(
    sunk: On<Sunk>,
    mut commands: Commands,
    art: Res<Art>,
    boats: Query<(), With<UBoat>>,
    banners: Query<Entity, With<Banner>>,
) {
    if !boats.contains(sunk.entity) {
        return;
    }
    clear(&mut commands, &banners);
    spawn_banner(
        &mut commands,
        &art,
        "U-Boot verloren".into(),
        "Zurück nach der Welle".into(),
    );
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
        if t > SLAM + HOLD + LEAVE {
            commands.entity(entity).despawn();
            continue;
        }
        let slam = (t / SLAM).min(1.0);
        let scale = 1.3 - 0.3 * (1.0 - (1.0 - slam).powi(3));
        let leave = ((t - SLAM - HOLD) / LEAVE).clamp(0.0, 1.0);
        for &child in children {
            if let Ok(mut transform) = sheets.get_mut(child) {
                transform.scale = Vec2::splat(scale);
                transform.translation = Val2::px(0.0, -300.0 * leave * leave);
            }
        }
    }
}
