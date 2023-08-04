use bevy::prelude::*;
use bevy_ggrs::Session;
use bevy_ggrs::ggrs::{PlayerHandle, PlayerType, SessionBuilder};
use bevy_matchbox::prelude::*;

use crate::{
    AppState, FontAssets, GGRSConfig, BUTTON_TEXT, FPS, HOVERED_BUTTON, INPUT_DELAY,
    MAX_PREDICTION, NORMAL_BUTTON, NUM_PLAYERS, PRESSED_BUTTON,
};

//const MATCHBOX_ADDR: &str = "ws://127.0.0.1:3536";
const MATCHBOX_ADDR: &str = "wss://match.gschup.dev";

#[derive(Component)]
pub struct MenuConnectUI;

#[derive(Component)]
pub enum MenuConnectBtn {
    Back,
}

#[derive(Resource)]
pub struct LocalHandles {
    pub handles: Vec<PlayerHandle>,
}

#[derive(Resource)]
pub struct ConnectData {
    pub lobby_id: String,
}

pub fn create_matchbox_socket(
    mut commands: Commands,
    connect_data: Res<ConnectData>,
) {
    let lobby_id = &connect_data.lobby_id;
    let room_url = format!("{MATCHBOX_ADDR}/{lobby_id}");
    let socket: MatchboxSocket<SingleChannel> = MatchboxSocket::new_ggrs(room_url);
    commands.insert_resource(Some(socket));
    commands.remove_resource::<ConnectData>();
}

pub fn update_matchbox_socket(
    commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut socket_res: ResMut<Option<MatchboxSocket<SingleChannel>>>,
) {
    if let Some(socket) = socket_res.as_mut() {
        socket.accept_new_connections();
        if socket.players().len() >= NUM_PLAYERS {
            // take the socket
            let socket = socket_res.as_mut().take().unwrap();
            create_ggrs_session(commands, socket);
            state
                .set(AppState::RoundOnline)
                .expect("Could not change state.");
        }
    }
}

pub fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Option<MatchboxSocket<SingleChannel>>>();
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // ui camera
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MenuConnectUI);

    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE,
            ..Default::default()
        })
        .with_children(|parent| {
            // lobby id display
            parent.spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                text: Text::with_section(
                    "Searching a match...",
                    TextStyle {
                        font: font_assets.default_font.clone(),
                        font_size: 32.,
                        color: BUTTON_TEXT,
                    },
                    Default::default(),
                ),
                ..Default::default()
            });

            // back button
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(16.)),
                        padding: Rect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON,
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Back to Menu",
                            TextStyle {
                                font: font_assets.default_font.clone(),
                                font_size: 40.0,
                                color: BUTTON_TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuConnectBtn::Back);
        })
        .insert(MenuConnectUI);
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuConnectBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
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
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuConnectBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
            match btn {
                MenuConnectBtn::Back => {
                    state
                        .set(AppState::MenuMain)
                        .expect("Could not change state.");
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuConnectUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn create_ggrs_session(mut commands: Commands, socket: MatchboxSocket<SingleChannel>) {
    // create a new ggrs session
    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(NUM_PLAYERS)
        .with_max_prediction_window(MAX_PREDICTION)
        .with_fps(FPS)
        .expect("Invalid FPS")
        .with_input_delay(INPUT_DELAY);

    // add players
    let mut handles = Vec::new();
    for (i, player_type) in socket.players().iter().enumerate() {
        if *player_type == PlayerType::Local {
            handles.push(i);
        }
        sess_build = sess_build
            .add_player(player_type.clone(), i)
            .expect("Invalid player added.");
    }

    // start the GGRS session
    let sess = sess_build
        .start_p2p_session(socket)
        .expect("Session could not be created.");

    commands.insert_resource(Session::P2PSession(sess));
    commands.insert_resource(LocalHandles { handles });
}
