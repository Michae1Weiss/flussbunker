//! Four paper notes (RM top right with tweened value and floating transactions, wave top
//! left, the boat's torpedoes and hull bottom left, the bunker's integrity bottom right next
//! to the bunker), the Tab overlay (ranges,
//! reloads, targeting, HP of damaged ships, torpedo lane, route, shelling reach) and enemy
//! health bars. Pictures where drawn, words only where not. Reads other features' state;
//! sole writer of `DisplayedMarks`, `Overlay`, `OverlayLabel`, `OverlayIcon`.
//! Rules: docs/design/ui-input.md

use bevy::prelude::*;

use crate::balance::BoatModel;
use crate::balance::{RM_TWEEN_SECONDS, Z_UI};
use crate::game::art::{paper_backing, Art, PaperTag};
use crate::game::bunker::Integrity;
use crate::game::combat::Health;
use crate::game::economy::Treasury;
use crate::game::enemy::{Enemy, Shelling};
use crate::game::input::ToggleOverlay;
use crate::game::map::RiverPath;
use crate::game::palette::{FAINT, GAIN, INK, NOTE, PENCIL, RED};
use crate::game::stroke::Strokes;
use crate::game::tower::{Gun, Range, Targeting, Tower};
use crate::game::uboat::{DryFire, Flotilla, Heading, Lane, Station, Torpedoes, Trim, UBoat};
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<DisplayedMarks>()
        .init_resource::<Overlay>()
        .add_systems(OnEnter(Screen::Gameplay), (spawn_notes, reset_marks))
        .add_systems(
            Update,
            (
                scale_ui,
                toggle_overlay,
                tween_marks,
                update_rm_note,
                update_wave_note,
                update_bunker,
                update_boat_note,
                flash_empty_tubes,
                float_transactions,
                draw_health_bars,
                draw_route,
                draw_overlay,
                attach_labels,
                update_labels,
                update_icons,
            )
                .chain()
                .in_set(Frame::Present),
        );
}

/// RM as shown: lags the treasury by `RM_TWEEN_SECONDS` and remembers the last real value
/// so a change can float up as a transaction.
#[derive(Resource, Default)]
pub struct DisplayedMarks {
    pub shown: f32,
    last: u32,
}

/// The tactical overlay, toggled with Tab. A mode, not a peek.
#[derive(Resource, Default)]
pub struct Overlay {
    pub on: bool,
}

/// A floating "+12" / "-60" on the RM note.
#[derive(Component)]
struct Transaction(Timer);

#[derive(Component)]
struct RmNote;

/// The paper behind the RM figure; transactions float from its edge.
#[derive(Component)]
struct RmPaper;

#[derive(Component)]
struct WaveNote;

#[derive(Component)]
struct BunkerText;

#[derive(Component)]
struct BoatName;

/// Shown only while the boat is under water.
#[derive(Component)]
struct DiveIcon;

/// The row of torpedoes and hull; hidden while the boat is sunk.
#[derive(Component)]
struct Gauges;

/// Torpedo tube `n`: inked while loaded, faint while empty, gone beyond the model's tubes.
#[derive(Component)]
struct TorpedoPip(u8);

#[derive(Component)]
struct HullText;

/// The targeting mode as a drawn icon under a turret in the overlay; replaces the text
/// label once the icon exists. Despawned with its turret.
#[derive(Component)]
pub struct OverlayIcon {
    subject: Entity,
}

/// World units below a turret's centre where its overlay label or icon sits.
const TOWER_LABEL_DROP: f32 = 26.0;

/// Overlay text that follows `subject`; despawned with it.
#[derive(Component)]
pub struct OverlayLabel {
    subject: Entity,
    offset: Vec2,
}

/// Logical window height the UI sizes are designed for.
const UI_REFERENCE_HEIGHT: f32 = 800.0;

/// A paper note pinned at `place`: the drawn paper if there is one, its content in a row
/// (or whatever `place` sets).
fn spawn_note(commands: &mut Commands, art: &Art, name: &'static str, place: Node) -> Entity {
    let backing = paper_backing(art, "ui/note.png");
    let mut paper = commands.spawn((
        Name::new(name),
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
            ..place
        },
    ));
    if backing.is_none() {
        paper.insert(BackgroundColor(NOTE));
    }
    let paper = paper.id();
    if let Some(backing) = backing {
        commands.spawn((backing, ChildOf(paper)));
    }
    paper
}

/// A drawn icon `size` pixels high in `parent`, if the drawing exists.
fn icon(
    commands: &mut Commands,
    art: &Art,
    key: &str,
    size: f32,
    parent: Entity,
    extra: impl Bundle,
) -> Option<Entity> {
    let image = art.image(key)?;
    Some(
        commands
            .spawn((
                ImageNode::new(image),
                Node {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    ..default()
                },
                extra,
                ChildOf(parent),
            ))
            .id(),
    )
}

fn text(commands: &mut Commands, size: f32, parent: Entity, marker: impl Bundle) {
    commands.spawn((
        marker,
        Text::new(String::new()),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(INK),
        ChildOf(parent),
    ));
}

fn row(commands: &mut Commands, gap: f32, parent: Entity, extra: impl Bundle) -> Entity {
    commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(gap),
                ..default()
            },
            extra,
            ChildOf(parent),
        ))
        .id()
}

fn spawn_notes(mut commands: Commands, art: Res<Art>) {
    let rm = spawn_note(
        &mut commands,
        &art,
        "RM note",
        Node {
            top: Val::Px(10.0),
            right: Val::Px(12.0),
            ..default()
        },
    );
    commands.entity(rm).insert(RmPaper);
    icon(&mut commands, &art, "ui/icons/coin.png", 31.0, rm, ());
    text(&mut commands, 26.0, rm, RmNote);

    let wave = spawn_note(
        &mut commands,
        &art,
        "Wave note",
        Node {
            top: Val::Px(10.0),
            left: Val::Px(12.0),
            ..default()
        },
    );
    text(&mut commands, 22.0, wave, WaveNote);

    let bunker = spawn_note(
        &mut commands,
        &art,
        "Bunker note",
        Node {
            bottom: Val::Px(10.0),
            right: Val::Px(12.0),
            ..default()
        },
    );
    icon(&mut commands, &art, "ui/icons/bunker.png", 28.0, bunker, ());
    text(&mut commands, 24.0, bunker, BunkerText);

    let boat = spawn_note(
        &mut commands,
        &art,
        "Boat note",
        Node {
            bottom: Val::Px(10.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        },
    );
    let name = row(&mut commands, 6.0, boat, ());
    text(&mut commands, 20.0, name, BoatName);
    icon(
        &mut commands,
        &art,
        "ui/icons/dive.png",
        22.0,
        name,
        DiveIcon,
    );
    let gauges = row(&mut commands, 1.0, boat, (Gauges, UiTransform::default()));
    let most_tubes = BoatModel::ALL.iter().map(|m| m.stats().tubes).max();
    for n in 0..most_tubes.unwrap_or(0) {
        icon(
            &mut commands,
            &art,
            "ui/icons/torpedo.png",
            22.0,
            gauges,
            TorpedoPip(n),
        );
    }
    commands.spawn((
        Node {
            width: Val::Px(12.0),
            ..default()
        },
        ChildOf(gauges),
    ));
    icon(&mut commands, &art, "ui/icons/hull.png", 22.0, gauges, ());
    text(&mut commands, 20.0, gauges, HullText);
}

/// Sizes are designed at 800 logical pixels high; larger windows scale the UI up.
fn scale_ui(window: Single<&Window, Changed<Window>>, mut scale: ResMut<UiScale>) {
    let wanted = (window.height() / UI_REFERENCE_HEIGHT).clamp(1.0, 2.0);
    if (scale.0 - wanted).abs() > 0.01 {
        scale.0 = wanted;
    }
}

fn reset_marks(mut displayed: ResMut<DisplayedMarks>, treasury: Res<Treasury>) {
    displayed.shown = treasury.marks as f32;
    displayed.last = treasury.marks;
}

fn toggle_overlay(mut intents: MessageReader<ToggleOverlay>, mut overlay: ResMut<Overlay>) {
    for _ in intents.read() {
        overlay.on = !overlay.on;
    }
}

fn tween_marks(
    time: Res<Time<Real>>,
    treasury: Res<Treasury>,
    mut displayed: ResMut<DisplayedMarks>,
    mut commands: Commands,
    notes: Query<Entity, With<RmPaper>>,
) {
    if treasury.marks != displayed.last {
        let delta = treasury.marks as i64 - displayed.last as i64;
        displayed.last = treasury.marks;
        for note in &notes {
            commands.spawn((
                Name::new("Transaction"),
                Transaction(Timer::from_seconds(1.2, TimerMode::Once)),
                DespawnOnExit(Screen::Gameplay),
                Text::new(format!("{delta:+}")),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(if delta > 0 { GAIN } else { RED }),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Percent(100.0),
                    top: Val::Px(6.0),
                    margin: UiRect::right(Val::Px(8.0)),
                    ..default()
                },
                ChildOf(note),
            ));
        }
    }
    let target = treasury.marks as f32;
    let rate = (time.delta_secs() / RM_TWEEN_SECONDS).min(1.0);
    displayed.shown += (target - displayed.shown) * rate;
    if (displayed.shown - target).abs() < 0.5 {
        displayed.shown = target;
    }
}

fn float_transactions(
    time: Res<Time<Real>>,
    mut commands: Commands,
    mut floating: Query<(Entity, &mut Transaction, &mut Node, &mut TextColor)>,
) {
    for (entity, mut transaction, mut node, mut color) in &mut floating {
        if transaction.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = transaction.0.fraction();
        node.top = Val::Px(6.0 - 26.0 * progress);
        color.0.set_alpha(1.0 - progress);
    }
}

/// With a drawn coin beside it the figure stands alone.
fn update_rm_note(
    mut note: Single<&mut Text, With<RmNote>>,
    displayed: Res<DisplayedMarks>,
    art: Res<Art>,
) {
    let marks = displayed.shown.round() as u32;
    note.0 = if art.image("ui/icons/coin.png").is_some() {
        marks.to_string()
    } else {
        format!("RM {marks}")
    };
}

fn update_wave_note(mut note: Single<&mut Text, With<WaveNote>>, wave: Res<WaveState>) {
    let wanted = wave.label();
    if note.0 != wanted {
        note.0 = wanted;
    }
}

fn update_bunker(
    mut note: Single<&mut Text, With<BunkerText>>,
    integrity: Res<Integrity>,
    art: Res<Art>,
) {
    let wanted = if art.image("ui/icons/bunker.png").is_some() {
        integrity.0.to_string()
    } else {
        format!("Bunker {}", integrity.0)
    };
    if note.0 != wanted {
        note.0 = wanted;
    }
}

fn update_boat_note(
    flotilla: Res<Flotilla>,
    art: Res<Art>,
    boats: Query<(&Health, &Torpedoes, &Trim), With<UBoat>>,
    mut texts: ParamSet<(
        Single<&mut Text, With<BoatName>>,
        Single<&mut Text, With<HullText>>,
    )>,
    mut nodes: Query<(&mut Node, Has<Gauges>, Has<DiveIcon>), Without<TorpedoPip>>,
    mut pips: Query<(&TorpedoPip, &mut ImageNode, &mut Node)>,
) {
    let stats = flotilla.model.stats();
    let boat = boats.single().ok();
    texts.p0().0 = match boat {
        Some(_) => stats.name.to_string(),
        None => format!("{} gesunken", stats.name),
    };
    let submerged = boat.is_some_and(|(_, _, trim)| !matches!(trim, Trim::Surfaced));
    for (mut node, gauges, dive) in &mut nodes {
        let shown = match (gauges, dive) {
            (true, _) => boat.is_some(),
            (_, true) => submerged,
            _ => continue,
        };
        node.display = if shown { Display::Flex } else { Display::None };
    }
    let Some((health, torpedoes, _)) = boat else {
        return;
    };

    let drawn_tubes = !pips.is_empty();
    for (pip, mut image, mut node) in &mut pips {
        let alpha = if pip.0 < torpedoes.loaded { 1.0 } else { 0.25 };
        image.color = Color::WHITE.with_alpha(alpha);
        node.display = if pip.0 < stats.tubes {
            Display::Flex
        } else {
            Display::None
        };
    }
    let hull = format!("{:.0}%", health.fraction() * 100.0);
    let mut words = Vec::new();
    if !drawn_tubes {
        words.push(format!("Torpedos {}/{}", torpedoes.loaded, stats.tubes));
    }
    if art.image("ui/icons/hull.png").is_none() {
        words.push(format!("Rumpf {hull}"));
    } else {
        words.push(hull);
    }
    let wanted = words.join("   ");
    let mut text = texts.p1();
    if text.0 != wanted {
        text.0 = wanted;
    }
}

/// Seconds the torpedo row shakes, washes red and shows its empty tubes after fire found
/// none loaded. A red tint would not show on black ink, hence the wash.
const DRY_FLASH: f32 = 0.5;

fn flash_empty_tubes(
    time: Res<Time<Real>>,
    mut dry: MessageReader<DryFire>,
    mut left: Local<f32>,
    mut row: Query<(&mut UiTransform, &mut BackgroundColor), With<Gauges>>,
    mut pips: Query<(&TorpedoPip, &mut ImageNode)>,
    boats: Query<&Torpedoes, With<UBoat>>,
) {
    if dry.read().count() > 0 {
        *left = DRY_FLASH;
    }
    *left = (*left - time.delta_secs()).max(0.0);
    let strength = *left / DRY_FLASH;
    for (mut transform, mut wash) in &mut row {
        let shake = (time.elapsed_secs() * 70.0).sin() * 4.0 * strength;
        transform.translation = Val2::px(shake, 0.0);
        wash.0 = RED.with_alpha(0.35 * strength);
    }
    if strength == 0.0 {
        return;
    }
    let loaded = boats.single().map_or(0, |t| t.loaded);
    for (pip, mut image) in &mut pips {
        if pip.0 >= loaded {
            image.color = Color::WHITE.with_alpha(0.25 + 0.75 * strength);
        }
    }
}

fn draw_health_bars(
    mut strokes: ResMut<Strokes>,
    boats: Query<(&Transform, &Health), With<Enemy>>,
) {
    for (transform, health) in &boats {
        let fraction = health.fraction();
        if fraction >= 1.0 {
            continue;
        }
        let base = transform.translation.truncate() + Vec2::new(-14.0, -20.0);
        strokes.line(base, base + Vec2::X * 28.0, FAINT);
        strokes.line(base, base + Vec2::X * 28.0 * fraction, RED);
    }
}

/// The enemy route: always faintly in the prep phase, otherwise only in the overlay.
fn draw_route(
    mut strokes: ResMut<Strokes>,
    path: Res<RiverPath>,
    wave: Res<WaveState>,
    overlay: Res<Overlay>,
) {
    if !(overlay.on || (!wave.running && !wave.prologue)) {
        return;
    }
    let ink = FAINT;
    let mut s = 0.0;
    let dash = 18.0;
    while s < path.length() {
        strokes.line(
            path.point_at(s),
            path.point_at((s + dash * 0.6).min(path.length())),
            ink,
        );
        s += dash;
    }
}

fn draw_overlay(
    mut strokes: ResMut<Strokes>,
    overlay: Res<Overlay>,
    path: Res<RiverPath>,
    towers: Query<(&Transform, &Range, &Gun), With<Tower>>,
    shellers: Query<(&Transform, &Shelling), With<Enemy>>,
    boats: Query<(&Transform, &Station, &Heading), With<UBoat>>,
) {
    if !overlay.on {
        return;
    }
    let ink = PENCIL;
    for (transform, range, gun) in &towers {
        let at = transform.translation.truncate();
        strokes.circle(at, range.0, ink);
        strokes.arc(at, 20.0, std::f32::consts::TAU * gun.reload.fraction(), INK);
    }
    for (transform, shelling) in &shellers {
        strokes.circle(
            transform.translation.truncate(),
            shelling.range,
            RED.with_alpha(0.5),
        );
    }
    for (transform, station, heading) in &boats {
        let lane = Lane::new(&path, transform.translation.truncate(), station, heading);
        strokes.line(lane.from, lane.from + lane.axis * lane.length, RED);
    }
}

/// Every enemy and turret gets one overlay label when it appears; the boat's state is on
/// its note.
fn attach_labels(
    mut commands: Commands,
    enemies: Query<Entity, Added<Enemy>>,
    towers: Query<Entity, Added<Tower>>,
) {
    let spawn = |commands: &mut Commands, subject: Entity, offset: Vec2| {
        commands.spawn((
            Name::new("Overlay label"),
            DespawnOnExit(Screen::Gameplay),
            OverlayLabel { subject, offset },
            Text2d::new(String::new()),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(INK),
            PaperTag,
            Visibility::Hidden,
            Transform::from_translation(Vec3::new(0.0, 0.0, Z_UI)),
        ));
    };
    for enemy in &enemies {
        spawn(&mut commands, enemy, Vec2::new(0.0, 24.0));
    }
    for tower in &towers {
        spawn(&mut commands, tower, Vec2::new(0.0, -TOWER_LABEL_DROP));
        commands.spawn((
            Name::new("Overlay icon"),
            DespawnOnExit(Screen::Gameplay),
            OverlayIcon { subject: tower },
            Sprite {
                custom_size: Some(Vec2::splat(18.0)),
                ..default()
            },
            Visibility::Hidden,
            Transform::from_translation(Vec3::new(0.0, 0.0, Z_UI)),
        ));
    }
}

fn targeting_icon(targeting: Targeting) -> &'static str {
    match targeting {
        Targeting::First => "ui/icons/first.png",
        Targeting::Strongest => "ui/icons/strongest.png",
        Targeting::Weakest => "ui/icons/weakest.png",
        Targeting::Nearest => "ui/icons/nearest.png",
    }
}

fn update_icons(
    overlay: Res<Overlay>,
    art: Res<Art>,
    mut commands: Commands,
    mut icons: Query<(
        Entity,
        &OverlayIcon,
        &mut Sprite,
        &mut Visibility,
        &mut Transform,
    )>,
    towers: Query<(&Transform, &Targeting), (With<Tower>, Without<OverlayIcon>)>,
) {
    for (entity, icon, mut sprite, mut visibility, mut transform) in &mut icons {
        let Ok((tower, targeting)) = towers.get(icon.subject) else {
            commands.entity(entity).despawn();
            continue;
        };
        let image = art.image(targeting_icon(*targeting));
        *visibility = if overlay.on && image.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if let Some(image) = image {
            if sprite.image != image {
                sprite.image = image;
            }
        }
        transform.translation =
            (tower.translation.truncate() - Vec2::Y * TOWER_LABEL_DROP).extend(Z_UI);
    }
}

fn update_labels(
    overlay: Res<Overlay>,
    art: Res<Art>,
    mut commands: Commands,
    mut labels: Query<(
        Entity,
        &OverlayLabel,
        &mut Text2d,
        &mut Visibility,
        &mut Transform,
    )>,
    subjects: Query<&Transform, Without<OverlayLabel>>,
    enemies: Query<&Health, With<Enemy>>,
    towers: Query<&Targeting, With<Tower>>,
) {
    for (entity, label, mut text, mut visibility, mut transform) in &mut labels {
        let Ok(subject) = subjects.get(label.subject) else {
            commands.entity(entity).despawn();
            continue;
        };
        // An undamaged ship says nothing: full HP is not news.
        let wanted = if let Ok(health) = enemies.get(label.subject) {
            if health.current < health.max {
                format!("{:.0}/{:.0}", health.current, health.max)
            } else {
                String::new()
            }
        } else if let Ok(targeting) = towers.get(label.subject) {
            if art.image(targeting_icon(*targeting)).is_some() {
                String::new()
            } else {
                targeting.label().to_string()
            }
        } else {
            String::new()
        };
        if text.0 != wanted {
            text.0 = wanted;
        }
        *visibility = if overlay.on && !text.0.is_empty() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = (subject.translation.truncate() + label.offset).extend(Z_UI);
    }
}
