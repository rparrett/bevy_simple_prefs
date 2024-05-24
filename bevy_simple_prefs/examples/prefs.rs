use bevy::{color::palettes::tailwind, ecs::system::EntityCommands, prelude::*};
use bevy_simple_prefs::{Prefs, PrefsPlugin};

// All `Prefs` must also be `Reflect` and `Default`.
#[derive(Prefs, Reflect, Default)]
struct ExamplePrefs {
    // Each field of the `Prefs` will be inserted into the `App` as a separate `Resource`.
    volume: Volume,
    difficulty: Difficulty,
}

// All `Prefs` fields must be `Resource`, `Reflect`, and `Clone`.
#[derive(Resource, Reflect, Clone, Eq, PartialEq)]
struct Volume(pub u32);
impl Default for Volume {
    fn default() -> Self {
        Self(50)
    }
}

#[derive(Resource, Reflect, Clone, Eq, PartialEq, Debug)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}
impl Default for Difficulty {
    fn default() -> Self {
        Self::Normal
    }
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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PrefsPlugin::<ExamplePrefs>::default()))
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

fn volume_buttons(
    up_query: Query<&Interaction, (Changed<Interaction>, With<VolumeUpButton>)>,
    down_query: Query<&Interaction, (Changed<Interaction>, With<VolumeDownButton>)>,
    mut volume: ResMut<Volume>,
) {
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
        text.sections[0].value.clone_from(&format!("{}", volume.0));
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
        text.sections[0]
            .value
            .clone_from(&format!("{:?}", *difficulty));
    }
}

fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut UiImage),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut image) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                image.color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                image.color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                image.color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            build_row(parent).with_children(|parent| {
                build_header(parent, "Volume".to_string());
            });

            build_row(parent).with_children(|parent| {
                build_button(parent, "<".to_string(), VolumeDownButton);
                build_label(parent, "".to_string(), VolumeLabel);
                build_button(parent, ">".to_string(), VolumeUpButton);
            });

            build_row(parent).with_children(|parent| {
                build_header(parent, "Difficulty".to_string());
            });

            build_row(parent).with_children(|parent| {
                build_button(parent, "<".to_string(), DifficultyDownButton);
                build_label(parent, "".to_string(), DifficultyLabel);
                build_button(parent, ">".to_string(), DifficultyUpButton);
            });
        });
}

fn build_header(parent: &mut ChildBuilder, text: String) {
    parent.spawn(TextBundle::from_section(
        text,
        TextStyle {
            font_size: TEXT_SIZE,
            color: TEXT_COLOR.into(),
            ..default()
        },
    ));
}

fn build_row<'a>(parent: &'a mut ChildBuilder) -> EntityCommands<'a> {
    parent.spawn(NodeBundle {
        style: Style {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(5.),
            ..default()
        },
        ..default()
    })
}

fn build_label<M: Component>(parent: &mut ChildBuilder, text: String, marker: M) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_radius: BorderRadius::all(Val::Px(5.)),
            background_color: LABEL_BACKGROUND.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    text,
                    TextStyle {
                        font_size: TEXT_SIZE,
                        color: TEXT_COLOR.into(),
                        ..default()
                    },
                ),
                marker,
            ));
        });
}

fn build_button<M: Component>(parent: &mut ChildBuilder, text: String, marker: M) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(50.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                border_radius: BorderRadius::all(Val::Px(5.)),
                image: UiImage::default().with_color(NORMAL_BUTTON.into()),
                ..default()
            },
            marker,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: TEXT_SIZE,
                    color: BUTTON_TEXT_COLOR.into(),
                    ..default()
                },
            ));
        });
}
