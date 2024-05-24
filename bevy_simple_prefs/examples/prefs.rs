use bevy::prelude::*;
use bevy_simple_prefs::{Prefs, PrefsPlugin};

#[derive(Resource, Reflect, Default, Clone)]
struct A {
    val: u32,
}
#[derive(Resource, Reflect, Default, Clone)]
struct B {
    val: u32,
}

#[derive(Reflect, Prefs, Default)]
struct ExampleStruct {
    a: A,
    b: B,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PrefsPlugin::<ExampleStruct>::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (changed, loaded, button_system))
        .run();
}

fn changed(a: Res<A>) {
    if a.is_changed() {
        info!("a is changed! {}", a.val);
    }
}

fn loaded(status: Res<PrefsStatus<ExampleStruct>>) {
    if status.is_changed() && status.loaded {
        info!("Loaded!");
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiImage),
        (Changed<Interaction>, With<Button>),
    >,
    mut a: ResMut<A>,
) {
    for (interaction, mut image) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                image.color = PRESSED_BUTTON;
                a.val += 1;
            }
            Interaction::Hovered => {
                image.color = HOVERED_BUTTON;
            }
            Interaction::None => {
                image.color = NORMAL_BUTTON;
            }
        }
    }
}

fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    image: UiImage::default().with_color(NORMAL_BUTTON),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}
