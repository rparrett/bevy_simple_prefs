use bevy::app::Update;

#[test]
fn test() {
    use bevy::app::App;
    use bevy::ecs::system::Resource;
    use bevy::prelude::Reflect;
    use bevy::prelude::World;
    use bevy_simple_prefs::Prefs;

    #[derive(Resource, Reflect, Clone, Default)]
    struct A {
        val: u32,
    }
    #[derive(Resource, Reflect, Clone, Default)]
    struct B {
        val: u32,
    }

    #[derive(Reflect, Prefs, Default)]
    struct ExampleStruct {
        a: A,
        b: B,
    }

    let mut app = App::new();
    app.add_systems(Update, ExampleStruct::save);
}
