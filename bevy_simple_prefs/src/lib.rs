use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        component::Component,
        system::{Commands, Query, Resource},
        world::{CommandQueue, World},
    },
    log::warn,
    reflect::Reflect,
    tasks::{block_on, futures_lite::future, Task},
};
pub use bevy_simple_prefs_derive::*;

pub trait Prefs {
    fn init(app: &mut App);
    fn save(world: &mut World);
    fn load(world: &mut World);
}

#[derive(Default)]
pub struct PrefsPlugin<T> {
    pub settings: PrefsSettings<T>,
}

#[derive(Resource)]
pub struct PrefsSettings<T> {
    pub filename: String,
    pub _phantom: PhantomData<T>,
}

impl<T> Clone for PrefsSettings<T> {
    fn clone(&self) -> Self {
        Self {
            filename: self.filename.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for PrefsSettings<T> {
    fn default() -> Self {
        Self {
            filename: "prefs.ron".to_string(),
            _phantom: Default::default(),
        }
    }
}

#[derive(Resource)]
pub struct PrefsStatus<T> {
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

#[derive(Component)]
pub struct LoadPrefsTask(pub Task<CommandQueue>);

impl<T: Prefs + Reflect> Plugin for PrefsPlugin<T> {
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
