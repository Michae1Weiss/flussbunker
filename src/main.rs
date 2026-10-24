use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
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
                .set(ImagePlugin::default_linear())
                // No `.meta` files ship; asking for each one is a 404 per asset on the web.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run()
}
