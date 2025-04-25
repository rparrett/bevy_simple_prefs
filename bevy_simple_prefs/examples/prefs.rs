//! Example showing typical usage of `PrefsPlugin`.

use bevy::{color::palettes::tailwind, log::LogPlugin, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin};

// All `Prefs` must also be `Reflect` and `Default`.
#[derive(Prefs, Reflect, Default)]
struct ExamplePrefs {
    // Each field of the `Prefs` will be inserted into the `App` as a separate `Resource`.
    volume: Volume,
    difficulty: Difficulty,
}

// All `Prefs` fields must be `Resource`, `Reflect`, and `Clone`.
#[derive(Resource, Reflect, Clone, Eq, PartialEq, Debug)]
struct Volume(pub u32);

#[derive(Resource, Reflect, Clone, Eq, PartialEq, Debug, Default)]
enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "prefs=debug,bevy_simple_prefs=debug".into(),
            ..default()
        }))
        // `PrefsPlugin` is generic, add as many different types of
        // prefs as you want. Make sure to set a unique filename for
        // each.
        .add_plugins(PrefsPlugin::<ExamplePrefs>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                volume_buttons,
                volume_label.run_if(resource_changed::<Volume>),
                difficulty_buttons,
                difficulty_label.run_if(resource_changed::<Difficulty>),
                button_style,
            ),
        )
        .run();
}

const TEXT_SIZE: f32 = 40.;
const TEXT_COLOR: Srgba = tailwind::EMERALD_50;
const BUTTON_TEXT_COLOR: Srgba = tailwind::EMERALD_50;
const LABEL_BACKGROUND: Srgba = tailwind::EMERALD_800;
const NORMAL_BUTTON: Srgba = tailwind::EMERALD_500;
const HOVERED_BUTTON: Srgba = tailwind::EMERALD_600;
const PRESSED_BUTTON: Srgba = tailwind::EMERALD_700;

#[derive(Component)]
struct VolumeUpButton;
#[derive(Component)]
struct VolumeDownButton;
#[derive(Component)]
struct VolumeLabel;

#[derive(Component)]
struct DifficultyUpButton;
#[derive(Component)]
struct DifficultyDownButton;
#[derive(Component)]
struct DifficultyLabel;

fn volume_buttons(
    up_query: Query<&Interaction, (Changed<Interaction>, With<VolumeUpButton>)>,
    down_query: Query<&Interaction, (Changed<Interaction>, With<VolumeDownButton>)>,
    mut volume: ResMut<Volume>,
) {
    // Prefs are persisted when the individual resources are changed.
    // Be careful when mutably dereferencing, as this can trigger unnecessary IO.
    let current = volume.bypass_change_detection().0;

    for _ in up_query.iter().filter(|i| **i == Interaction::Pressed) {
        let new = (current + 10).min(100);
        volume.set_if_neq(Volume(new));
    }
    for _ in down_query.iter().filter(|i| **i == Interaction::Pressed) {
        let new = current.saturating_sub(10);
        volume.set_if_neq(Volume(new));
    }
}

fn volume_label(volume: Res<Volume>, mut text_query: Query<&mut Text, With<VolumeLabel>>) {
    for mut text in &mut text_query {
        text.0.clone_from(&format!("{}", volume.0));
    }
}

fn difficulty_buttons(
    up_query: Query<&Interaction, (Changed<Interaction>, With<DifficultyUpButton>)>,
    down_query: Query<&Interaction, (Changed<Interaction>, With<DifficultyDownButton>)>,
    mut difficulty: ResMut<Difficulty>,
) {
    for _ in up_query.iter().filter(|i| **i == Interaction::Pressed) {
        let next = difficulty.bypass_change_detection().next();
        difficulty.set_if_neq(next);
    }
    for _ in down_query.iter().filter(|i| **i == Interaction::Pressed) {
        let prev = difficulty.bypass_change_detection().prev();
        difficulty.set_if_neq(prev);
    }
}

fn difficulty_label(
    difficulty: Res<Difficulty>,
    mut text_query: Query<&mut Text, With<DifficultyLabel>>,
) {
    for mut text in &mut text_query {
        text.0.clone_from(&format!("{:?}", *difficulty));
    }
}

fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.),
            ..default()
        },
        children![
            (row(), children![header("Volume".to_string())]),
            (
                row(),
                children![
                    (button("<".to_string()), VolumeDownButton),
                    label("".to_string(), VolumeLabel),
                    (button(">".to_string()), VolumeUpButton),
                ],
            ),
            (row(), children![header("Difficulty".to_string())]),
            (
                row(),
                children![
                    (button("<".to_string()), DifficultyDownButton),
                    label("".to_string(), DifficultyLabel),
                    (button(">".to_string()), DifficultyUpButton),
                ],
            )
        ],
    ));
}

fn header(text: String) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: TEXT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR.into()),
    )
}

fn row() -> impl Bundle {
    Node {
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(5.),
        ..default()
    }
}

fn label<M: Component>(text: String, text_marker: M) -> impl Bundle {
    (
        Node {
            width: Val::Px(150.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderRadius::all(Val::Px(5.)),
        BackgroundColor(LABEL_BACKGROUND.into()),
        children![(
            Text::new(text),
            TextFont {
                font_size: TEXT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR.into()),
            text_marker,
        )],
    )
}

fn button(text: String) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(50.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderRadius::all(Val::Px(5.)),
        BackgroundColor(NORMAL_BUTTON.into()),
        children![(
            Text::new(text),
            TextFont {
                font_size: TEXT_SIZE,
                ..default()
            },
            TextColor(BUTTON_TEXT_COLOR.into()),
        )],
    )
}

impl Difficulty {
    fn next(&self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Hard,
            Self::Hard => Self::Hard,
        }
    }
    fn prev(&self) -> Self {
        match self {
            Self::Easy => Self::Easy,
            Self::Normal => Self::Easy,
            Self::Hard => Self::Normal,
        }
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self(50)
    }
}
