//! Paper menu parts shared by the screens and the HUD: a veil over the map, a card, headings
//! and buttons. A button inks red and grows a little under the cursor, taps on hover and
//! stamps on press; what it does is an observer of `Pointer<Click>`.

use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;

use crate::game::art::{paper_backing, Art};
use crate::game::audio::Speaker;
use crate::game::palette::{INK, NOTE, PAPER, RED};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, highlight);
}

#[derive(Clone, Copy)]
pub enum Size {
    Big,
    Normal,
    Small,
}

impl Size {
    /// Minimum width in UI pixels, and the font size.
    fn metrics(self) -> (f32, f32) {
        match self {
            Size::Big => (300.0, 46.0),
            Size::Normal => (240.0, 30.0),
            Size::Small => (44.0, 30.0),
        }
    }
}

/// The text on a button, inked by `highlight`.
#[derive(Component)]
pub struct Label;

/// A button that is the obvious next step: inked red even at rest.
#[derive(Component)]
pub struct Accent;

/// A full-screen column over a veil of paper, through which the map shows faintly.
pub fn veil(commands: &mut Commands, name: &'static str, scope: impl Bundle) -> Entity {
    commands
        .spawn((
            Name::new(name),
            scope,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(PAPER.with_alpha(0.65)),
            GlobalZIndex(10),
        ))
        .id()
}

/// A sheet of paper in `parent`, holding a column of items.
pub fn card(commands: &mut Commands, art: &Art, parent: Entity) -> Entity {
    let backing = paper_backing(art, "ui/banner.png");
    let mut card = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            padding: UiRect::axes(Val::Px(56.0), Val::Px(36.0)),
            ..default()
        },
        ChildOf(parent),
    ));
    if backing.is_none() {
        card.insert(BackgroundColor(NOTE));
    }
    let card = card.id();
    if let Some(backing) = backing {
        commands.spawn((backing, Pickable::IGNORE, ChildOf(card)));
    }
    card
}

pub fn heading(commands: &mut Commands, parent: Entity, text: &str, size: f32) -> Entity {
    commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(INK),
            Pickable::IGNORE,
            ChildOf(parent),
        ))
        .id()
}

pub fn button<E: EntityEvent, B: Bundle, M>(
    commands: &mut Commands,
    art: &Art,
    parent: Entity,
    label: &str,
    size: Size,
    action: impl IntoObserverSystem<E, B, M>,
) -> Entity {
    let (width, font) = size.metrics();
    let backing = paper_backing(art, "ui/note.png");
    let mut button = commands.spawn((
        Name::new("Button"),
        Button,
        Node {
            min_width: Val::Px(width),
            padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        UiTransform::default(),
        ChildOf(parent),
    ));
    if backing.is_none() {
        button.insert(BackgroundColor(NOTE));
    }
    button
        .observe(action)
        .observe(|_: On<Pointer<Over>>, mut speaker: Speaker| speaker.play("ui_hover"))
        .observe(|_: On<Pointer<Press>>, mut speaker: Speaker| speaker.play("ui_click"));
    let button = button.id();
    if let Some(backing) = backing {
        commands.spawn((backing, Pickable::IGNORE, ChildOf(button)));
    }
    commands.spawn((
        Label,
        Text::new(label),
        TextFont {
            font_size: FontSize::Px(font),
            ..default()
        },
        TextColor(INK),
        Pickable::IGNORE,
        ChildOf(button),
    ));
    button
}

/// A button under the cursor grows by this much.
pub const HOT_SCALE: f32 = 1.06;

fn highlight(
    mut buttons: Query<
        (&Interaction, &mut UiTransform, &Children, Has<Accent>),
        (Changed<Interaction>, With<Button>),
    >,
    mut labels: Query<&mut TextColor, With<Label>>,
) {
    for (interaction, mut transform, children, accent) in &mut buttons {
        let hot = *interaction != Interaction::None;
        transform.scale = Vec2::splat(if hot { HOT_SCALE } else { 1.0 });
        for &child in children {
            if let Ok(mut color) = labels.get_mut(child) {
                color.0 = if hot || accent { RED } else { INK };
            }
        }
    }
}
