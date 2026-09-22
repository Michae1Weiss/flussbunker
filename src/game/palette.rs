//! The paper palette: every overlay, label and note draws with these, so the UI reads on
//! a hand-drawn paper map. Red belongs to the player side and to danger.

use bevy::prelude::*;

pub const PAPER: Color = Color::srgb(0.96, 0.95, 0.92);
/// The paper notes: slightly yellowed, so they sit apart from the map.
pub const NOTE: Color = Color::srgb(0.93, 0.88, 0.72);
pub const INK: Color = Color::srgb(0.16, 0.13, 0.10);
pub const PENCIL: Color = Color::srgba(0.16, 0.13, 0.10, 0.55);
pub const FAINT: Color = Color::srgba(0.16, 0.13, 0.10, 0.28);
pub const RED: Color = Color::srgb(0.78, 0.22, 0.18);
pub const GAIN: Color = Color::srgb(0.20, 0.45, 0.15);
