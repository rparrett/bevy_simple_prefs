//! Bevy Simple Prefs
//!
//! A small Bevy plugin for persisting multiple `Resource`s to a single file.

use std::{any::TypeId, marker::PhantomData, path::PathBuf};

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        component::Component,
        system::{Commands, Query},
        world::{CommandQueue, World},
    },
    log::warn,
    prelude::{IntoScheduleConfigs, Resource},
    reflect::{
        GetTypeRegistration, Reflect, TypePath, TypeRegistry,
        serde::{TypedReflectDeserializer, TypedReflectSerializer},
    },
    tasks::{Task, block_on, futures_lite::future},
};
pub use bevy_simple_prefs_derive::*;
use ron::ser::{PrettyConfig, to_string_pretty};
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
pub struct PrefsPlugin<T: Reflect + TypePath> {
    /// Path to the file where the preferences will be stored.
    ///
    /// This value is not used in Wasm builds.
    ///
    /// Defaults to `(crate name of T)_prefs.ron` in the current working directory.
    pub path: PathBuf,
    /// String to use for the key when storing preferences in localStorage on
    /// Wasm builds.
    ///
    /// This value should be unique to your app to avoid collisions with other
    /// apps on the same web server. On itch.io, for example, many other games
    /// will be using the same storage area.
    ///
    /// Defaults to `(crate name of T)::(type name of T).ron`.
    pub local_storage_key: String,
    /// PhantomData
    pub _phantom: PhantomData<T>,
}
impl<T: Reflect + TypePath> Default for PrefsPlugin<T> {
    fn default() -> Self {
        let package_name = T::crate_name().unwrap_or("bevy_simple");
        let file_name = format!("{}_prefs.ron", package_name);

        Self {
            path: file_name.into(),
            // For Wasm, we want to provide a unique name for a project by default
            // to avoid collisions when doing local development or deploying multiple
            // apps to the same web server.
            local_storage_key: format!("{package_name}::{}.ron", T::short_type_path()),
            _phantom: Default::default(),
        }
    }
}

/// Settings for [`PrefsPlugin`].
#[derive(Resource)]
pub struct PrefsSettings<T> {
    /// See [`PrefsPlugin::local_storage_key`].
    pub local_storage_key: String,
    /// See [`PrefsPlugin::path`].
    pub path: PathBuf,
    /// PhantomData
    pub _phantom: PhantomData<T>,
}

/// Current status of the [`PrefsPlugin`].
#[derive(Resource)]
pub struct PrefsStatus<T> {
    /// `true` if the preferences have been loaded
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
        app.insert_resource::<PrefsSettings<T>>(PrefsSettings {
            path: self.path.clone(),
            local_storage_key: self.local_storage_key.clone(),
            _phantom: Default::default(),
        });
        app.init_resource::<PrefsStatus<T>>();

        <T>::init(app);

        // `save` checks load status and needs to run in the same frame after `handle_tasks`.
        app.add_systems(Update, (handle_tasks, <T>::save).chain());
        app.add_systems(Startup, <T>::load);
    }
}

fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut LoadPrefsTask>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            bevy::log::debug!("adding pref resource update commands");
            commands.append(&mut commands_queue);
        }
    }
}

/// Loads preferences from persisted data.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_str(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Loads preferences from persisted data.
#[cfg(target_arch = "wasm32")]
pub fn load_str(local_storage_key: &str) -> Option<String> {
    let Some(window) = web_sys::window() else {
        warn!("Failed to load save file: no window.");
        return None;
    };

    let Ok(Some(storage)) = window.local_storage() else {
        warn!("Failed to load save file: no storage.");
        return None;
    };

    let Ok(maybe_item) = storage.get_item(local_storage_key) else {
        warn!("Failed to load save file: failed to get item.");
        return None;
    };

    maybe_item
}

/// Persists preferences.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_str(path: &std::path::Path, data: &str) {
    if let Err(e) = std::fs::write(path, data) {
        warn!("Failed to store save file: {:?}", e);
    }
}

/// Persists preferences.
#[cfg(target_arch = "wasm32")]
pub fn save_str(local_storage_key: &str, data: &str) {
    let Some(window) = web_sys::window() else {
        warn!("Failed to store save file: no window.");
        return;
    };

    let Ok(Some(storage)) = window.local_storage() else {
        warn!("Failed to store save file: no storage.");
        return;
    };

    if let Err(e) = storage.set_item(local_storage_key, data) {
        warn!("Failed to store save file: {:?}", e);
    }
}

/// Deserializes preferences
pub fn deserialize<T: Reflect + GetTypeRegistration + Default>(
    serialized: &str,
) -> Result<T, ron::de::Error> {
    let mut registry = TypeRegistry::new();
    registry.register::<T>();
    let registration = registry.get(TypeId::of::<T>()).unwrap();

    let mut deserializer = ron::Deserializer::from_str(serialized).unwrap();

    let de = TypedReflectDeserializer::new(registration, &registry);
    let dynamic_struct = de.deserialize(&mut deserializer)?;

    let mut val = T::default();
    val.apply(&*dynamic_struct);
    Ok(val)
}

/// Serialize preferences
pub fn serialize<T: Reflect + GetTypeRegistration>(to_save: &T) -> Result<String, ron::Error> {
    let mut registry = TypeRegistry::new();
    registry.register::<T>();

    let config = PrettyConfig::default();
    let reflect_serializer = TypedReflectSerializer::new(to_save, &registry);
    to_string_pretty(&reflect_serializer, config)
}
