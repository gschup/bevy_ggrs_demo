mod checksum;
mod menu;
mod round;

use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use bevy_ggrs::GGRSPlugin;
use checksum::{checksum_players, Checksum};
use ggrs::Config;
use menu::{
    connect::{create_matchbox_socket, update_matchbox_socket},
    online::{update_lobby_btn, update_lobby_id, update_lobby_id_display},
};
use round::{
    apply_inputs, check_win, increase_frame_count, move_players, print_p2p_events, setup_round,
    spawn_players, update_velocity, FrameCount, Velocity,
};

const NUM_PLAYERS: usize = 2;
const FPS: usize = 60;
const ROLLBACK_SYSTEMS: &str = "rollback_systems";
const CHECKSUM_UPDATE: &str = "checksum_update";
const MAX_PREDICTION: usize = 12;
const INPUT_DELAY: usize = 2;
const CHECK_DISTANCE: usize = 2;

const DISABLED_BUTTON: Color = Color::rgb(0.8, 0.5, 0.5);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    AssetLoading,
    MenuMain,
    MenuOnline,
    MenuConnect,
    RoundLocal,
    RoundOnline,
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

#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub default_font: Handle<Font>,
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
        .continue_to_state(AppState::MenuMain)
        .with_collection::<ImageAssets>()
        .with_collection::<FontAssets>()
        .build(&mut app);

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(round::input)
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
        .add_state(AppState::AssetLoading)
        // main menu
        .add_system_set(SystemSet::on_enter(AppState::MenuMain).with_system(menu::main::setup_ui))
        .add_system_set(
            SystemSet::on_update(AppState::MenuMain)
                .with_system(menu::main::btn_visuals)
                .with_system(menu::main::btn_listeners),
        )
        .add_system_set(SystemSet::on_exit(AppState::MenuMain).with_system(menu::main::cleanup_ui))
        //online menu
        .add_system_set(
            SystemSet::on_enter(AppState::MenuOnline).with_system(menu::online::setup_ui),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MenuOnline)
                .with_system(update_lobby_id)
                .with_system(update_lobby_id_display)
                .with_system(update_lobby_btn)
                .with_system(menu::online::btn_visuals)
                .with_system(menu::online::btn_listeners),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::MenuOnline).with_system(menu::online::cleanup_ui),
        )
        // connect menu
        .add_system_set(
            SystemSet::on_enter(AppState::MenuConnect)
                .with_system(create_matchbox_socket)
                .with_system(menu::connect::setup_ui),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MenuConnect)
                .with_system(update_matchbox_socket)
                .with_system(menu::connect::btn_visuals)
                .with_system(menu::connect::btn_listeners),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::MenuConnect)
                .with_system(menu::connect::cleanup)
                .with_system(menu::connect::cleanup_ui),
        )
        // win menu
        .add_system_set(SystemSet::on_enter(AppState::Win).with_system(menu::win::setup_ui))
        .add_system_set(
            SystemSet::on_update(AppState::Win)
                .with_system(menu::win::btn_visuals)
                .with_system(menu::win::btn_listeners),
        )
        .add_system_set(SystemSet::on_exit(AppState::Win).with_system(menu::win::cleanup_ui))
        // local round
        .add_system_set(
            SystemSet::on_enter(AppState::RoundLocal)
                .with_system(setup_round)
                .with_system(spawn_players),
        )
        .add_system_set(SystemSet::on_update(AppState::RoundLocal).with_system(check_win))
        .add_system_set(SystemSet::on_exit(AppState::RoundLocal).with_system(round::cleanup))
        // online round
        .add_system_set(
            SystemSet::on_enter(AppState::RoundOnline)
                .with_system(setup_round)
                .with_system(spawn_players),
        )
        .add_system_set(
            SystemSet::on_update(AppState::RoundOnline)
                .with_system(print_p2p_events)
                .with_system(check_win),
        )
        .add_system_set(SystemSet::on_exit(AppState::RoundOnline).with_system(round::cleanup));

    #[cfg(target_arch = "wasm32")]
    {
        app.add_system(bevy_web_resizer::web_resize_system);
    }

    app.run();
}
