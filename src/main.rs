use bevy::{prelude::*, window::WindowResolution};
use flussbunker::GamePlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Flussbunker".to_string(),
                        resolution: WindowResolution::new(1280, 720),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_linear()),
        )
        .add_plugins(GamePlugin)
        .run()
}
