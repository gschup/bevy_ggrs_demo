use bevy::prelude::*;
use bevy_ggrs::SessionType;
use ggrs::{PlayerType, SessionBuilder};

use crate::{
    connect::LocalHandles, AppState, GGRSConfig, ImageAssets, CHECK_DISTANCE, FPS, HOVERED_BUTTON,
    INPUT_DELAY, MAX_PREDICTION, NORMAL_BUTTON, NUM_PLAYERS, PRESSED_BUTTON, TEXT,
};

#[derive(Component)]
pub struct MenuUI;

#[derive(Component)]
pub struct OnlineMatchBtn;

#[derive(Component)]
pub struct LocalMatchBtn;

pub fn setup_menu(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    asset_server: Res<AssetServer>,
) {
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(MenuUI);

    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect::all(Val::Px(0.)),
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // logo
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Px(500.0), Val::Px(139.0)),
                    margin: Rect::all(Val::Px(16.)),
                    padding: Rect::all(Val::Px(16.)),
                    ..Default::default()
                },
                image: image_assets.ggrs_logo.clone().into(),
                ..Default::default()
            });
            // quick match button
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(16.)),
                        padding: Rect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Online",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(OnlineMatchBtn);

            // local mode button
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(16.)),
                        padding: Rect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Local",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(LocalMatchBtn);
        })
        .insert(MenuUI);
}

pub fn update_online_match_btn(
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<OnlineMatchBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state
                    .set(AppState::Connect)
                    .expect("Could not change state.");
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

pub fn update_local_match_btn(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<LocalMatchBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                create_synctest_session(&mut commands);
                state
                    .set(AppState::LocalRound)
                    .expect("Could not change state.");
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

fn create_synctest_session(commands: &mut Commands) {
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
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

    commands.insert_resource(sess);
    commands.insert_resource(SessionType::SyncTestSession);
    commands.insert_resource(LocalHandles {
        handles: (0..NUM_PLAYERS).collect(),
    });
}

pub fn cleanup_menu_ui(query: Query<Entity, With<MenuUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
