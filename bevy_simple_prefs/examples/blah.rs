use bevy::prelude::*;
use bevy_simple_prefs::{Preferences, PreferencesPlugin};

#[derive(Resource, Reflect, Default, Clone)]
struct A {
    val: u32,
}
#[derive(Resource, Reflect, Default, Clone)]
struct B {
    val: u32,
}

#[derive(Reflect, Preferences, Default)]
struct ExampleStruct {
    a: A,
    b: B,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(PreferencesPlugin::<ExampleStruct>::default());
    app.add_systems(Update, changed);
    app.run();
}

fn changed(a: Res<A>) {
    if a.is_changed() {
        info!("a is changed! {}", a.val);
    }
}
