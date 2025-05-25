//! Example demonstrating how to store the preferences in the user's home directory.

use bevy::{log::LogPlugin, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin, PrefsStatus};

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context;

#[derive(Resource, Reflect, Default, Clone)]
struct Launches(u32);

#[derive(Reflect, Prefs, Default)]
struct ExamplePrefs {
    launches: Launches,
}

fn main() -> anyhow::Result<()> {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "dirs=debug,bevy_simple_prefs=debug".into(),
                ..default()
            }),
            PrefsPlugin::<ExamplePrefs> {
                // This example uses the `dirs` crate to locate the user's "local config
                // directory", e.g. `C:\Users\Alice\AppData\Local` on Windows, or
                // `/usr/home/alice/.config` on Linux.
                #[cfg(not(target_arch = "wasm32"))]
                path: {
                    let dir = dirs::config_local_dir()
                        .context("Determining local config directory")?
                        .join("home_dir");
                    std::fs::create_dir_all(&dir).context("Creating prefs directory")?;
                    dir.join("prefs.ron")
                },
                ..default()
            },
        ))
        .add_systems(
            Update,
            print.run_if(condition_changed_to(
                true,
                |status: Res<PrefsStatus<ExamplePrefs>>| status.loaded,
            )),
        )
        .run();

    Ok(())
}

fn print(mut launches: ResMut<Launches>) {
    info!("Launches: {}", launches.0);
    launches.0 += 1;
}
