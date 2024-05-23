use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{system::Resource, world::World},
    log::warn,
    reflect::Reflect,
};
pub use bevy_simple_prefs_derive::*;

pub trait Preferences {
    fn init(app: &mut App);
    fn save(world: &mut World);
    fn load(world: &mut World);
}

#[derive(Default)]
pub struct PreferencesPlugin<T> {
    pub settings: PreferencesSettings<T>,
}

#[derive(Resource)]
pub struct PreferencesSettings<T> {
    pub filename: String,
    _phantom: PhantomData<T>,
}

impl<T> Clone for PreferencesSettings<T> {
    fn clone(&self) -> Self {
        Self {
            filename: self.filename.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for PreferencesSettings<T> {
    fn default() -> Self {
        Self {
            filename: "preferences.ron".to_string(),
            _phantom: Default::default(),
        }
    }
}

impl<T: Preferences + Reflect> Plugin for PreferencesPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.settings.clone());

        <T>::init(app);

        app.add_systems(Update, <T>::save);
        app.add_systems(Startup, <T>::load);
    }
}

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
