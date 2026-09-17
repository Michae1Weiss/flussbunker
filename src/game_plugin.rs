//! Composition root: adds the feature plugins and declares system ordering once.

use bevy::prelude::*;

use crate::{
    game,
    screens::{self, Screen},
};

pub struct GamePlugin;

/// Simulation order, declared once here; no system uses `.after`. Runs in `FixedUpdate`
/// at 64 Hz so gameplay is frame-rate independent; input is sampled in `Update`
/// (`just_pressed` is per frame) and reaches the simulation as messages.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sim {
    Spawn,
    Movement,
    Targeting,
    Damage,
    Death,
}

/// Per-frame work in `Update`: input sampling, then drawing.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Frame {
    Input,
    Present,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.09)))
            .insert_resource(Time::<Fixed>::from_hz(64.0))
            .add_systems(Startup, spawn_camera);

        app.configure_sets(
            FixedUpdate,
            (
                Sim::Spawn,
                Sim::Movement,
                Sim::Targeting,
                Sim::Damage,
                Sim::Death,
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
        app.configure_sets(
            Update,
            (Frame::Input, Frame::Present)
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );

        app.add_plugins((screens::plugin, game::plugin));

        #[cfg(feature = "dev")]
        app.add_plugins(crate::dev_tools::plugin);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}
