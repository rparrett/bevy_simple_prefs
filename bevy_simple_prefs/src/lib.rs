//! Bevy Simple Prefs
//!
//! A small Bevy plugin for persisting multiple `Resource`s to a single file.

use std::{
    any::TypeId,
    marker::PhantomData,
    path::{Path, PathBuf},
};

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
    /// Filename (or LocalStorage key) for the preferences file.
    pub filename: String,
    /// Path to the directory where the preferences file will be stored.
    ///
    /// This value is not used in WASM builds.
    pub path: PathBuf,
    /// PhantomData
    pub _phantom: PhantomData<T>,
}
impl<T: Reflect + TypePath> Default for PrefsPlugin<T> {
    fn default() -> Self {
        // For wasm, we want to provide a unique name for a project by default
        // to avoid collisions when doing local development or deploying multiple
        // apps to the same web server (for example, itch.io).
        let package_name = T::crate_name().unwrap_or("bevy_simple");

        Self {
            filename: format!("{}_prefs.ron", package_name),
            path: Default::default(),
            _phantom: Default::default(),
        }
    }
}

/// Settings for `PrefsPlugin`.
#[derive(Resource)]
pub struct PrefsSettings<T> {
    /// Filename (or LocalStorage key) for the preferences file.
    pub filename: String,
    /// Path to the directory where the preferences file will be stored.
    pub path: PathBuf,
    /// PhantomData
    pub _phantom: PhantomData<T>,
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
        app.insert_resource::<PrefsSettings<T>>(PrefsSettings {
            filename: self.filename.clone(),
            path: self.path.clone(),
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
pub fn load_str(dir: &Path, filename: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = dir.join(filename);

        std::fs::read_to_string(path).ok()
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
pub fn save_str(dir: &Path, filename: &str, data: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = dir.join(filename);

        if let Err(e) = std::fs::write(path, data) {
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
