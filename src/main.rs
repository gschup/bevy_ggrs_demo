mod connect;
mod menu;
mod round;

use crate::{connect::*, menu::*, round::*};
use bevy::prelude::*;
use bevy_ggrs::GGRSPlugin;
use ggrs::Config;

const NUM_PLAYERS: usize = 2;
const FPS: usize = 60;
const ROLLBACK_SYSTEMS: &str = "rollback_systems";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Menu,
    Connect,
    Round,
    Win,
}

#[derive(SystemLabel, Debug, Clone, Hash, Eq, PartialEq)]
enum Systems {
    Inp,
}

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = round::Input;
    type State = u8;
    type Address = String;
}

fn main() {
    let mut app = App::new();
    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input)
        .register_rollback_type::<Transform>()
        .register_rollback_type::<Velocity>()
        .register_rollback_type::<FrameCount>()
        .with_rollback_schedule(
            Schedule::default().with_stage(
                ROLLBACK_SYSTEMS,
                SystemStage::parallel()
                    .with_system(apply_inputs.label(Systems::Inp))
                    .with_system(move_players.after(Systems::Inp))
                    .with_system(increase_frame_count),
            ),
        )
        .build(&mut app);

    app.add_plugins(DefaultPlugins)
        .add_state(AppState::Menu)
        // menu
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(menu))
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu))
        // connection
        .add_system_set(SystemSet::on_enter(AppState::Connect).with_system(setup_connect))
        .add_system_set(SystemSet::on_update(AppState::Connect).with_system(connect))
        .add_system_set(SystemSet::on_exit(AppState::Connect).with_system(create_ggrs_session))
        // round
        .add_system_set(
            SystemSet::on_enter(AppState::Round)
                .with_system(setup_round)
                .with_system(spawn_players),
        )
        .add_system_set(SystemSet::on_update(AppState::Round).with_system(stats))
        .add_system_set(SystemSet::on_exit(AppState::Round).with_system(cleanup_round))
        .run();
}
