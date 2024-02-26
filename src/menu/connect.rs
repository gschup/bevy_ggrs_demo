use bevy::prelude::*;
use bevy_ggrs::ggrs::{PlayerType, SessionBuilder};
use bevy_ggrs::{LocalPlayers, Session};
use bevy_matchbox::prelude::*;

use crate::{
    AppState, FontAssets, GGRSConfig, BUTTON_TEXT, FPS, HOVERED_BUTTON, INPUT_DELAY,
    MAX_PREDICTION, NORMAL_BUTTON, NUM_PLAYERS, PRESSED_BUTTON,
};

const MATCHBOX_ADDR: &str = "ws://127.0.0.1:3536";
// const MATCHBOX_ADDR: &str = "wss://match.gschup.dev";

#[derive(Component)]
pub struct MenuConnectUI;

#[derive(Component)]
pub enum MenuConnectBtn {
    Back,
}

#[derive(Resource)]
pub struct ConnectData {
    pub lobby_id: String,
}

pub fn create_matchbox_socket(mut commands: Commands, connect_data: Res<ConnectData>) {
    let lobby_id = &connect_data.lobby_id;
    let room_url = format!("{MATCHBOX_ADDR}/{lobby_id}");
    let socket: MatchboxSocket<SingleChannel> = MatchboxSocket::new_ggrs(room_url);
    commands.insert_resource(socket);
    commands.remove_resource::<ConnectData>();
}

pub fn update_matchbox_socket(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
) {
    // regularly call update_peers to update the list of connected peers
    for (peer, new_state) in socket.update_peers() {
        // you can also handle the specific dis(connections) as they occur:
        match new_state {
            PeerState::Connected => info!("peer {peer} connected"),
            PeerState::Disconnected => info!("peer {peer} disconnected"),
        }
    }

    if socket.players().len() >= NUM_PLAYERS {
        // create a new ggrs session
        let mut sess_build = SessionBuilder::<GGRSConfig>::new()
            .with_num_players(NUM_PLAYERS)
            .with_max_prediction_window(MAX_PREDICTION)
            .expect("Invalid prediction window")
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
        let channel = socket.take_channel(0).unwrap();
        let sess = sess_build
            .start_p2p_session(channel)
            .expect("Session could not be created.");

        // insert session as resource and switch state
        commands.insert_resource(Session::P2P(sess));
        commands.insert_resource(LocalPlayers(handles));
        state.set(AppState::RoundOnline);
    }
}

pub fn cleanup(mut _commands: Commands) {
    // FIXME: Removing MatchboxSocket crashes the game
    // commands.remove_resource::<MatchboxSocket<SingleChannel>>();
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // ui camera
    commands
        .spawn(Camera2dBundle::default())
        .insert(MenuConnectUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // lobby id display
            parent.spawn(TextBundle::from_section(
                "Searching a match...",
                TextStyle {
                    font: font_assets.default_font.clone(),
                    font_size: 32.,
                    color: BUTTON_TEXT,
                },
            ));
            // back button
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
                        "Back to Menu",
                        TextStyle {
                            font: font_assets.default_font.clone(),
                            font_size: 40.0,
                            color: BUTTON_TEXT,
                        },
                    ));
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
    mut state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuConnectBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuConnectBtn::Back => {
                    state.set(AppState::MenuMain);
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