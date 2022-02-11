use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_ggrs::SessionType;
use ggrs::SessionBuilder;
use matchbox_socket::WebRtcNonBlockingSocket;

use crate::{AppState, GGRSConfig, FPS, NUM_PLAYERS};

const MATCHBOX_ADDR: &str = "wss://match.gschup.dev";
const MAX_PREDICTION: usize = 12;
const INPUT_DELAY: usize = 2;

pub fn setup_connect(mut commands: Commands, task_pool: Res<IoTaskPool>) {
    let room_id = "random";
    let room_url = format!("{MATCHBOX_ADDR}/{room_id}");
    let (socket, message_loop) = WebRtcNonBlockingSocket::new(room_url);
    task_pool.spawn(message_loop).detach();
    commands.insert_resource(Some(socket));
}

pub fn connect(
    mut state: ResMut<State<AppState>>,
    mut socket: ResMut<Option<WebRtcNonBlockingSocket>>,
) {
    if let Some(socket) = socket.as_mut() {
        socket.accept_new_connections();
        if socket.players().len() >= NUM_PLAYERS {
            state.set(AppState::Round).expect("Could not change state.");
        }
    }
}

pub fn create_ggrs_session(
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcNonBlockingSocket>>,
) {
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
    for (i, player_type) in socket.players().iter().enumerate() {
        sess_build = sess_build
            .add_player(player_type.clone(), i)
            .expect("Invalid player added.");
    }

    // start the GGRS session
    let sess = sess_build
        .start_p2p_session(socket)
        .expect("Session could not be created.");

    commands.insert_resource(sess);
    commands.insert_resource(SessionType::P2PSession);
}
