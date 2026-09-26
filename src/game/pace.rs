//! The run's pace, as things to click and see instead of keys to learn: a next-wave button
//! and a 1x/2x button bottom centre, clear of the bunker (Enter and F still work), and in the
//! build phase the coming ships at the river mouth. Sole writer of `Preview`.

use bevy::prelude::*;

use crate::balance::{HULL_BEAM_RATIO, TILE_SIZE, WAVES, Z_NOTE};
use crate::game::art::{enemy_key, Art, PaperTag};
use crate::game::input::{StartWave, ToggleSpeed};
use crate::game::map::tile_to_world;
use crate::game::palette::{INK, PENCIL};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::widget::{button, Accent, Label, Size, HOT_SCALE};
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_buttons)
        .add_systems(
            Update,
            (update_buttons, show_preview).in_set(Frame::Present),
        );
}

#[derive(Component)]
struct NextWave;

#[derive(Component)]
struct Speed;

/// The ships of wave `.0` waiting at the river mouth.
#[derive(Component)]
struct Preview(usize);

/// Where the first ship of the preview lies, in tiles: on land just below the river mouth.
const PREVIEW_TILE: (f32, f32) = (1.5, 5.0);
/// Tiles between the preview's ship groups.
const PREVIEW_STEP: f32 = 1.9;
/// A preview ship's length in world units: all readable, bigger ships still bigger.
fn preview_length(size: f32) -> f32 {
    30.0 + 0.6 * size
}

fn key_hint(commands: &mut Commands, parent: Entity, key: &str) {
    commands.spawn((
        Text::new(key),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(PENCIL),
        Pickable::IGNORE,
        ChildOf(parent),
    ));
}

fn spawn_buttons(mut commands: Commands, art: Res<Art>) {
    let bar = commands
        .spawn((
            Name::new("Pace buttons"),
            DespawnOnExit(Screen::Gameplay),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                column_gap: Val::Px(10.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let speed = button(
        &mut commands,
        &art,
        bar,
        "1×",
        Size::Small,
        |_: On<Pointer<Click>>, mut speed: MessageWriter<ToggleSpeed>| {
            speed.write(ToggleSpeed);
        },
    );
    key_hint(&mut commands, speed, "F");
    commands.entity(speed).insert(Speed);
    let next = button(
        &mut commands,
        &art,
        bar,
        "",
        Size::Normal,
        |_: On<Pointer<Click>>, mut start: MessageWriter<StartWave>| {
            start.write(StartWave);
        },
    );
    key_hint(&mut commands, next, "Enter");
    commands.entity(next).insert((NextWave, Accent));
}

fn in_prep(wave: &WaveState) -> Option<usize> {
    (!wave.prologue && !wave.running && wave.index < WAVES.len()).then_some(wave.index)
}

/// The next-wave button breathes, one breath every this many seconds, while it waits.
const BREATH: f32 = 1.6;

fn update_buttons(
    wave: Res<WaveState>,
    time: Res<Time<Virtual>>,
    real: Res<Time<Real>>,
    mut buttons: Query<
        (
            &mut Node,
            &mut UiTransform,
            &Interaction,
            &Children,
            Has<NextWave>,
        ),
        Or<(With<NextWave>, With<Speed>)>,
    >,
    mut labels: Query<&mut Text, With<Label>>,
) {
    for (mut node, mut transform, interaction, children, next) in &mut buttons {
        let wanted = if next {
            let Some(index) = in_prep(&wave) else {
                node.display = Display::None;
                continue;
            };
            node.display = Display::Flex;
            let breath = (real.elapsed_secs() * std::f32::consts::TAU / BREATH).sin();
            transform.scale = Vec2::splat(match interaction {
                Interaction::None => 1.0 + 0.04 * breath,
                _ => HOT_SCALE,
            });
            format!("Welle {} »", index + 1)
        } else if time.relative_speed() > 1.0 {
            "2×".to_string()
        } else {
            "1×".to_string()
        };
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                if text.0 != wanted {
                    text.0.clone_from(&wanted);
                }
            }
        }
    }
}

fn show_preview(
    mut commands: Commands,
    art: Res<Art>,
    wave: Res<WaveState>,
    previews: Query<(Entity, &Preview)>,
) {
    let wanted = in_prep(&wave);
    let mut shown = false;
    for (entity, preview) in &previews {
        if Some(preview.0) == wanted {
            shown = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some(index) = wanted.filter(|_| !shown) else {
        return;
    };
    let root = commands
        .spawn((
            Name::new("Wave preview"),
            Preview(index),
            DespawnOnExit(Screen::Gameplay),
            Transform::from_translation(
                tile_to_world(PREVIEW_TILE.0, PREVIEW_TILE.1).extend(Z_NOTE),
            ),
            Visibility::default(),
        ))
        .id();
    for (i, group) in WAVES[index].groups.iter().enumerate() {
        let stats = group.kind.stats();
        let at = Vec3::X * i as f32 * PREVIEW_STEP * TILE_SIZE;
        // Bow downstream, like the ships about to come.
        let ship = Sprite {
            image: art.image(enemy_key(group.kind)).unwrap_or_default(),
            color: if art.image(enemy_key(group.kind)).is_some() {
                Color::WHITE
            } else {
                Color::srgb(stats.rgb[0], stats.rgb[1], stats.rgb[2])
            },
            custom_size: Some(Vec2::new(HULL_BEAM_RATIO, 1.0) * preview_length(stats.size)),
            ..default()
        };
        commands.spawn((
            ship,
            Transform::from_translation(at)
                .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
            ChildOf(root),
        ));
        commands.spawn((
            Text2d::new(format!("{}×", group.count)),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(INK),
            PaperTag,
            Transform::from_translation(at + Vec3::new(0.0, -38.0, 0.5)),
            ChildOf(root),
        ));
    }
}
