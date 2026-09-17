//! Dev-only diagnostics: frame timings and schedule ambiguity warnings.
//! `cargo run --features dev`

#![cfg(feature = "dev")]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
    ));

    app.edit_schedule(FixedUpdate, |schedule| {
        schedule.set_build_settings(bevy::ecs::schedule::ScheduleBuildSettings {
            ambiguity_detection: bevy::ecs::schedule::LogLevel::Warn,
            ..default()
        });
    });
}
