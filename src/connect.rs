use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_ggrs::SessionType;
use ggrs::{PlayerHandle, PlayerType, SessionBuilder};
use matchbox_socket::WebRtcSocket;

use crate::{AppState, GGRSConfig, FPS, INPUT_DELAY, MAX_PREDICTION, NUM_PLAYERS, TEXT};

const MATCHBOX_ADDR: &str = "ws://127.0.0.1:3536";

#[derive(Component)]
pub struct ConnectUI;

pub struct LocalHandles {
    pub handles: Vec<PlayerHandle>,
}

pub fn setup_connect(mut commands: Commands, task_pool: Res<IoTaskPool>) {
    let room_url = format!("{MATCHBOX_ADDR}/fighter?next=2");
    let (socket, message_loop) = WebRtcSocket::new(room_url);
    task_pool.spawn(message_loop).detach();
    commands.insert_resource(Some(socket));
}

pub fn connect(mut state: ResMut<State<AppState>>, mut socket: ResMut<Option<WebRtcSocket>>) {
    if let Some(socket) = socket.as_mut() {
        socket.accept_new_connections();
        if socket.players().len() >= NUM_PLAYERS {
            state
                .set(AppState::OnlineRound)
                .expect("Could not change state.");
        }
    }
}

pub fn create_ggrs_session(mut commands: Commands, mut socket: ResMut<Option<WebRtcSocket>>) {
    // take the socket
    let socket = socket
        .as_mut()
        .take()
        .expect("Should not leave the connecting state without an existing socket.");

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

    commands.insert_resource(sess);
    commands.insert_resource(LocalHandles { handles });
    commands.insert_resource(SessionType::P2PSession);
}

pub fn setup_connect_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(ConnectUI);
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                text: Text::with_section(
                    "Waiting for opponent...",
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 96.,
                        color: TEXT,
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .insert(ConnectUI);
}

pub fn cleanup_connect_ui(query: Query<Entity, With<ConnectUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
