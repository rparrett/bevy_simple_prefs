//! Example demonstrating how to store the preferences in the user's home directory.

use bevy::{log::LogPlugin, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin};

#[derive(Resource, Reflect, Default, Clone)]
struct Launches(u32);

#[derive(Reflect, Prefs, Default)]
struct ExamplePrefs {
    launches: Launches,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "home_dir=debug,bevy_simple_prefs=debug".into(),
                ..default()
            }),
            PrefsPlugin::<ExamplePrefs> {
                // This value won't be used in WASM builds
                path: home::home_dir().unwrap_or_default(),
                // Setting this is optional. `(your_package_name)_prefs.ron` will be used by default.
                filename: "custom_filename.ron".to_string(),
                ..default()
            },
        ))
        .add_systems(Update, print)
        .run();
}

fn print(launches: ResMut<Launches>) {
    if launches.is_changed() && !launches.is_added() {
        info!("Launches: {}", launches.0);
    }
}
