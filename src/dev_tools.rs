//! Dev-only tooling. Compiled out of release builds entirely.
//!
//! `cargo run --features dev`
//!
//! Kept deliberately thin: frame timings and schedule ambiguity detection.
//! `bevy-inspector-egui` is the obvious next addition — check its version
//! compatibility with Bevy 0.19 before pinning it, the ecosystem lags releases
//! by a few weeks.

#![cfg(feature = "dev")]

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
    ));

    // Ambiguity detection: warns when two systems touch the same data with no
    // ordering between them. In a game where damage, death and spawning all
    // run in the same tick, this catches real nondeterminism — the kind of bug
    // that reproduces once a week and never in a debugger.
    app.edit_schedule(FixedUpdate, |schedule| {
        schedule.set_build_settings(bevy::ecs::schedule::ScheduleBuildSettings {
            ambiguity_detection: bevy::ecs::schedule::LogLevel::Warn,
            ..default()
        });
    });
}
