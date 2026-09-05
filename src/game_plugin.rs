//! Composition root.
//!
//! Two jobs, and deliberately no third:
//!   1. Add the feature plugins.
//!   2. Declare system ordering **once**, as sets.
//!
//! No gameplay logic lives here. If you ever feel like adding a system to this
//! file, it belongs in a feature module instead.

use bevy::prelude::*;

use crate::{
    game,
    screens::{self, Screen},
};

pub struct GamePlugin;

/// The single ordering contract for the whole simulation.
///
/// Why a set enum instead of `.after(...)` sprinkled around: ordering is a
/// *global* property. Expressed here it is one readable list; expressed as
/// pairwise constraints across twelve files it is a graph nobody can hold in
/// their head. Adding a system later means picking a set, not re-deriving the
/// order.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sim {
    /// Wave spawning, tower construction, u-boat launches.
    Spawn,
    /// Everything that changes a position.
    Movement,
    /// Range checks, target selection, trigger pulls.
    Targeting,
    /// Damage messages are consumed and health is written.
    Damage,
    /// Health <= 0 is turned into despawns and payouts.
    Death,
}

/// Frame-rate-dependent work: input sampling and drawing.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Frame {
    /// Reads `ButtonInput`. Must be in `Update` — see the input gotcha below.
    Input,
    /// HUD text, gizmo overlays, anything purely visual.
    Present,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.09)))
            // 64 Hz simulation. Deterministic and independent of the monitor.
            .insert_resource(Time::<Fixed>::from_hz(64.0))
            .add_systems(Startup, spawn_camera);

        // Gate gameplay once, at the set level. Individual systems then never
        // repeat `.run_if(in_state(...))` — a whole class of "I forgot the run
        // condition" bugs simply cannot occur.
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
    // `Name` costs nothing and makes the inspector readable. Do it for every
    // top-level entity.
    commands.spawn((Name::new("Camera"), Camera2d));
}
