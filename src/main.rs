mod checksum;
mod connect;
mod menu;
mod round;
mod win;

use crate::{checksum::*, connect::*, menu::*, round::*, win::*};
use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_ggrs::GGRSPlugin;
use ggrs::Config;

#[cfg(target_arch = "wasm32")]
use approx::relative_eq;

const NUM_PLAYERS: usize = 2;
const FPS: usize = 60;
const ROLLBACK_SYSTEMS: &str = "rollback_systems";
const CHECKSUM_UPDATE: &str = "checksum_update";
const MAX_PREDICTION: usize = 12;
const INPUT_DELAY: usize = 2;
const CHECK_DISTANCE: usize = 2;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const TEXT: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    AssetLoading,
    Menu,
    Connect,
    LocalRound,
    OnlineRound,
    Win,
}

#[derive(SystemLabel, Debug, Clone, Hash, Eq, PartialEq)]
enum SystemLabel {
    Input,
    Velocity,
}

#[derive(AssetCollection)]
pub struct ImageAssets {
    #[asset(path = "images/ggrs_logo.png")]
    pub ggrs_logo: Handle<Image>,
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

    AssetLoader::new(AppState::AssetLoading)
        .continue_to_state(AppState::Menu)
        .with_collection::<ImageAssets>()
        .build(&mut app);

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input)
        .register_rollback_type::<Transform>()
        .register_rollback_type::<Velocity>()
        .register_rollback_type::<FrameCount>()
        .register_rollback_type::<Checksum>()
        .with_rollback_schedule(
            Schedule::default()
                .with_stage(
                    ROLLBACK_SYSTEMS,
                    SystemStage::parallel()
                        .with_system(apply_inputs.label(SystemLabel::Input))
                        .with_system(
                            update_velocity
                                .label(SystemLabel::Velocity)
                                .after(SystemLabel::Input),
                        )
                        .with_system(move_players.after(SystemLabel::Velocity))
                        .with_system(increase_frame_count),
                )
                .with_stage_after(
                    ROLLBACK_SYSTEMS,
                    CHECKSUM_UPDATE,
                    SystemStage::parallel().with_system(checksum_players),
                ),
        )
        .build(&mut app);

    app.add_plugins(DefaultPlugins)
        .add_system(update_window_size)
        .add_state(AppState::AssetLoading)
        // menu
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
        .add_system_set(
            SystemSet::on_update(AppState::Menu)
                .with_system(update_online_match_btn)
                .with_system(update_local_match_btn),
        )
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu_ui))
        // connection
        .add_system_set(
            SystemSet::on_enter(AppState::Connect)
                .with_system(setup_connect)
                .with_system(setup_connect_ui),
        )
        .add_system_set(SystemSet::on_update(AppState::Connect).with_system(connect))
        .add_system_set(
            SystemSet::on_exit(AppState::Connect)
                .with_system(create_ggrs_session)
                .with_system(cleanup_connect_ui),
        )
        // local round
        .add_system_set(
            SystemSet::on_enter(AppState::LocalRound)
                .with_system(setup_round)
                .with_system(spawn_players),
        )
        .add_system_set(SystemSet::on_update(AppState::LocalRound).with_system(check_win))
        .add_system_set(SystemSet::on_exit(AppState::LocalRound).with_system(cleanup_round))
        // online round
        .add_system_set(
            SystemSet::on_enter(AppState::OnlineRound)
                .with_system(setup_round)
                .with_system(spawn_players),
        )
        .add_system_set(
            SystemSet::on_update(AppState::OnlineRound)
                .with_system(print_p2p_events)
                .with_system(check_win),
        )
        .add_system_set(SystemSet::on_exit(AppState::OnlineRound).with_system(cleanup_round))
        // win screen
        .add_system_set(SystemSet::on_enter(AppState::Win).with_system(setup_win_ui))
        .add_system_set(SystemSet::on_update(AppState::Win).with_system(update_cont_btn))
        .add_system_set(SystemSet::on_exit(AppState::Win).with_system(cleanup_win_ui))
        .run();
}

#[allow(unused_variables, unused_mut)]
fn update_window_size(mut windows: ResMut<Windows>) {
    // TODO: use window resize event instead of polling
    #[cfg(target_arch = "wasm32")]
    {
        let web_window = web_sys::window().unwrap();
        let width = web_window.inner_width().unwrap().as_f64().unwrap() as f32 - 30.;
        let height = web_window.inner_height().unwrap().as_f64().unwrap() as f32 - 30.;
        let window = windows.get_primary_mut().unwrap();

        if relative_eq!(width, window.width()) && relative_eq!(height, window.height()) {
            return;
        }

        window.set_resolution(width, height);
    }
}
