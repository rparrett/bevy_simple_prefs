//! Example demonstrating how to keep track of the loading status of preferences.
//!
//! Prefs are initialized with their default values when an App starts, and updated
//! with the loaded values later. You may want to wait until they have been loaded
//! before running systems that rely on them.

use bevy::{log::LogPlugin, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin, PrefsStatus};

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum MyAppState {
    #[default]
    Loading,
    Playing,
}

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
                filter: "status=debug,bevy_simple_prefs=debug".into(),
                ..default()
            }),
            PrefsPlugin::<ExamplePrefs>::default(),
        ))
        .init_state::<MyAppState>()
        .add_systems(Update, check_status.run_if(in_state(MyAppState::Loading)))
        .add_systems(OnEnter(MyAppState::Playing), (print, increment).chain())
        .run();
}

fn check_status(status: Res<PrefsStatus<ExamplePrefs>>, mut next: ResMut<NextState<MyAppState>>) {
    if status.loaded {
        next.set(MyAppState::Playing);
    }
}

fn print(launches: Res<Launches>) {
    info!(
        "Loaded! This example has been launched {} time{}.",
        launches.0,
        if launches.0 != 1 { "s" } else { "" }
    )
}

fn increment(mut launches: ResMut<Launches>) {
    launches.0 += 1;
}
