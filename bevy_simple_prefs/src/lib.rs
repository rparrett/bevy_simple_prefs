//! Bevy Simple Prefs
//!
//! A small Bevy plugin for persisting multiple `Resource`s to a single file.

use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        component::Component,
        system::{Commands, Query, Resource},
        world::{CommandQueue, World},
    },
    log::warn,
    reflect::{serde::ReflectDeserializer, GetTypeRegistration, Reflect, TypePath, TypeRegistry},
    tasks::{block_on, futures_lite::future, Task},
};
pub use bevy_simple_prefs_derive::*;
use serde::de::DeserializeSeed;

/// A trait to be implemented by `bevy_simple_prefs_derive`.
pub trait Prefs {
    /// Runs when `PrefsPlugin` is built and initializes individual preference `Resource`s with default values.
    fn init(app: &mut App);
    /// Runs when individual preferences `Resources` are changed and persists preferences.
    fn save(world: &mut World);
    /// Loads preferences and updates individual preference `Resources`.
    fn load(world: &mut World);
}

/// The Bevy plugin responsible for persisting `T`.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_simple_prefs::{Prefs, PrefsPlugin};
///
/// #[derive(Prefs, Reflect, Default)]
/// struct ExamplePrefs {
///     difficulty: Difficulty,
/// }
///
/// #[derive(Resource, Reflect, Clone, Eq, PartialEq, Debug, Default)]
/// enum Difficulty {
///     Easy,
///     #[default]
///     Normal,
///     Hard,
/// }
///
/// App::new().add_plugins(PrefsPlugin::<ExamplePrefs>::default());
/// ```
#[derive(Default)]
pub struct PrefsPlugin<T: Reflect + TypePath> {
    /// Settings for `PrefsPlugin`.
    pub settings: PrefsSettings<T>,
}

/// Settings for `PrefsPlugin`.
#[derive(Resource)]
pub struct PrefsSettings<T> {
    /// Filename (or LocalStorage key) for the preferences file.
    pub filename: String,
    /// PhantomData
    pub _phantom: PhantomData<T>,
}

impl<T: Reflect + TypePath> Clone for PrefsSettings<T> {
    fn clone(&self) -> Self {
        Self {
            filename: self.filename.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Reflect + TypePath> Default for PrefsSettings<T> {
    fn default() -> Self {
        // For wasm, we want to provide a unique name for a project by default
        // to avoid collisions when doing local development or deploying multiple
        // apps to the same web server (for example, itch.io).
        let package_name = T::crate_name().unwrap_or("bevy_simple");

        Self {
            filename: format!("{}_prefs.ron", package_name),
            _phantom: Default::default(),
        }
    }
}

/// Current status of the `PrefsPlugin`.
#[derive(Resource)]
pub struct PrefsStatus<T> {
    /// `true` if the preferences have been
    pub loaded: bool,
    _phantom: PhantomData<T>,
}

impl<T> Default for PrefsStatus<T> {
    fn default() -> Self {
        Self {
            loaded: false,
            _phantom: Default::default(),
        }
    }
}

/// A component that holds the task responsible for updating individual preference `Resource`s after they have been loaded.
#[derive(Component)]
pub struct LoadPrefsTask(pub Task<CommandQueue>);

impl<T: Prefs + Reflect + TypePath> Plugin for PrefsPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.settings.clone());
        app.init_resource::<PrefsStatus<T>>();

        <T>::init(app);

        app.add_systems(Update, (<T>::save, handle_tasks));
        app.add_systems(Startup, <T>::load);
    }
}

fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut LoadPrefsTask>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut commands_queue);
        }
    }
}

/// Loads preferences from persisted data.
pub fn load_str(filename: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(filename).ok()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            warn!("Failed to load save file: no window.");
            return None;
        };

        let Ok(Some(storage)) = window.local_storage() else {
            warn!("Failed to load save file: no storage.");
            return None;
        };

        let Ok(maybe_item) = storage.get_item(filename) else {
            warn!("Failed to load save file: failed to get item.");
            return None;
        };

        maybe_item
    }
}

/// Persists preferences.
pub fn save_str(filename: &str, data: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = std::fs::write(filename, data) {
            warn!("Failed to store save file: {:?}", e);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                warn!("Failed to store save file: no window.");
                return;
            }
        };

        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => {
                warn!("Failed to store save file: no storage.");
                return;
            }
        };

        if let Err(e) = storage.set_item(filename, data) {
            warn!("Failed to store save file: {:?}", e);
        }
    }
}

/// Deserializes preferences
pub fn deserialize<T: Reflect + GetTypeRegistration + Default>(
    serialized: &str,
) -> Result<T, ron::de::Error> {
    let mut registry = TypeRegistry::new();
    registry.register::<T>();

    let mut deserializer = ron::Deserializer::from_str(serialized).unwrap();

    let de = ReflectDeserializer::new(&registry);
    let dynamic_struct = de.deserialize(&mut deserializer)?;

    let mut val = T::default();
    val.apply(&*dynamic_struct);
    Ok(val)
}
