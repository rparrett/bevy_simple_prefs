//! Regression coverage for async loading, change detection, and multiple plugins.

#![cfg(not(target_arch = "wasm32"))]

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_simple_prefs::{Prefs, PrefsPlugin, PrefsStatus, serialize};

#[derive(Resource, Reflect, Default, Clone)]
struct Volume(u32);

#[derive(Prefs, Reflect, Default)]
struct Settings {
    volume: Volume,
}

#[derive(Resource, Reflect, Default, Clone)]
struct Launches(u32);

#[derive(Prefs, Reflect, Default)]
struct Stats {
    launches: Launches,
}

fn update_until(app: &mut App, mut ready: impl FnMut(&App) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        if ready(app) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for preferences"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn loads_then_saves_changed_resources() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("bevy_simple_prefs-{}-{unique}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let settings_path = dir.join("settings.ron");
    let stats_path = dir.join("stats.ron");
    std::fs::write(&settings_path, "(volume: (25))").unwrap();
    std::fs::write(&stats_path, "(launches: (3))").unwrap();

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        PrefsPlugin::<Settings> {
            path: settings_path.clone(),
            ..default()
        },
        PrefsPlugin::<Stats> {
            path: stats_path.clone(),
            ..default()
        },
    ));
    update_until(&mut app, |app| {
        app.world().resource::<PrefsStatus<Settings>>().loaded
            && app.world().resource::<PrefsStatus<Stats>>().loaded
    });
    assert_eq!(app.world().resource::<Volume>().0, 25);
    assert_eq!(app.world().resource::<Launches>().0, 3);

    app.world_mut().resource_mut::<Volume>().0 = 75;
    app.world_mut().resource_mut::<Launches>().0 += 1;
    let expected_settings = serialize(&Settings { volume: Volume(75) }).unwrap();
    let expected_stats = serialize(&Stats {
        launches: Launches(4),
    })
    .unwrap();
    update_until(&mut app, |_| {
        std::fs::read_to_string(&settings_path).ok().as_deref() == Some(&expected_settings)
            && std::fs::read_to_string(&stats_path).ok().as_deref() == Some(&expected_stats)
    });
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}
