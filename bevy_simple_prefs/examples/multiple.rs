//! Example demonstrating how to store the preferences in the user's home directory.

use bevy::{input::common_conditions::input_just_pressed, log::LogPlugin, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin, PrefsStatus};

#[derive(Resource, Reflect, Default, Clone)]
struct Launches(u32);

#[derive(Reflect, Prefs, Default)]
struct ExampleStats {
    launches: Launches,
}

#[derive(Resource, Reflect, Default, Clone)]
struct Enabled(bool);

#[derive(Reflect, Prefs, Default)]
struct ExamplePrefs {
    enabled: Enabled,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "multiple=debug,bevy_simple_prefs=debug".into(),
                ..default()
            }),
            PrefsPlugin::<ExamplePrefs>::default(),
            PrefsPlugin::<ExampleStats> {
                path: "multiple_stats.ron".into(),
                ..default()
            },
        ))
        .add_systems(
            Update,
            print.run_if(condition_changed_to(
                true,
                |prefs_status: Res<PrefsStatus<ExamplePrefs>>,
                 stats_status: Res<PrefsStatus<ExampleStats>>| {
                    prefs_status.loaded && stats_status.loaded
                },
            )),
        )
        .add_systems(
            Update,
            toggle_enabled.run_if(input_just_pressed(KeyCode::Space)),
        )
        .run();
}

fn print(mut launches: ResMut<Launches>, enabled: Res<Enabled>) {
    info!("Launches: {}", launches.0);
    info!("Enabled: {} (space to toggle)", enabled.0);
    launches.0 += 1;
}

fn toggle_enabled(mut enabled: ResMut<Enabled>) {
    enabled.0 = !enabled.0;
    info!("Enabled: {}", enabled.0);
}
