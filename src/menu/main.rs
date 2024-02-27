use bevy::{app::AppExit, prelude::*};
use bevy_ggrs::ggrs::{PlayerType, SessionBuilder};
use bevy_ggrs::{LocalPlayers, Session};

use crate::{
    AppState, FontAssets, GGRSConfig, ImageAssets, BUTTON_TEXT, CHECK_DISTANCE, FPS,
    HOVERED_BUTTON, INPUT_DELAY, MAX_PREDICTION, NORMAL_BUTTON, NUM_PLAYERS, PRESSED_BUTTON,
};

#[derive(Component)]
pub struct MenuMainUI;

#[derive(Component)]
pub enum MenuMainBtn {
    OnlineMatch,
    LocalMatch,
    Quit,
}

pub fn setup_ui(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    font_assets: Res<FontAssets>,
) {
    // ui camera
    commands.spawn(Camera2dBundle::default()).insert(MenuMainUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // logo
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(500.0),
                    height: Val::Px(139.0),
                    margin: UiRect::all(Val::Px(16.)),
                    padding: UiRect::all(Val::Px(16.)),
                    ..Default::default()
                },
                image: image_assets.ggrs_logo.clone().into(),
                ..Default::default()
            });

            // online match button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Online",
                        TextStyle {
                            font: font_assets.default_font.clone(),
                            font_size: 40.0,
                            color: BUTTON_TEXT,
                        },
                    ));
                })
                .insert(MenuMainBtn::OnlineMatch);

            // local mode button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Local",
                        TextStyle {
                            font: font_assets.default_font.clone(),
                            font_size: 40.0,
                            color: BUTTON_TEXT,
                        },
                    ));
                })
                .insert(MenuMainBtn::LocalMatch);

            // quit button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(16.)),
                        padding: UiRect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Quit",
                        TextStyle {
                            font: font_assets.default_font.clone(),
                            font_size: 40.0,
                            color: BUTTON_TEXT,
                        },
                    ));
                })
                .insert(MenuMainBtn::Quit);
        })
        .insert(MenuMainUI);
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuMainBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
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

pub fn btn_listeners(
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuMainBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuMainBtn::OnlineMatch => {
                    state.set(AppState::MenuOnline);
                }
                MenuMainBtn::LocalMatch => {
                    create_synctest_session(&mut commands);
                    state.set(AppState::RoundLocal);
                }
                MenuMainBtn::Quit => {
                    exit.send(AppExit);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuMainUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn create_synctest_session(commands: &mut Commands) {
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
        .expect("Invalid prediction window")
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY)
        .with_check_distance(CHECK_DISTANCE);

    for i in 0..NUM_PLAYERS {
        sess_build = sess_build
            .add_player(PlayerType::Local, i)
            .expect("Could not add local player");
    }

    let sess = sess_build.start_synctest_session().expect("");

    commands.insert_resource(Session::SyncTest(sess));
    commands.insert_resource(LocalPlayers((0..NUM_PLAYERS).collect()));
}
