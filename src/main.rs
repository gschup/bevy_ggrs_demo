mod checksum;
mod menu;
mod round;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ggrs::ggrs::Config;
use bevy_ggrs::{GgrsApp, GgrsPlugin, GgrsSchedule, ReadInputs};
use bevy_matchbox::prelude::*;
use checksum::{checksum_players, Checksum};
use menu::{
    connect::{create_matchbox_socket, update_matchbox_socket},
    online::{update_lobby_btn, update_lobby_id, update_lobby_id_display},
};
use round::{
    apply_inputs, check_win, increase_frame_count, move_players, print_p2p_events, setup_round, spawn_players, update_velocity, FrameCount, Velocity
};

const NUM_PLAYERS: usize = 2;
const FPS: usize = 60;
const MAX_PREDICTION: usize = 12;
const INPUT_DELAY: usize = 2;
const CHECK_DISTANCE: usize = 2;

const DISABLED_BUTTON: Color = Color::rgb(0.8, 0.5, 0.5);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    AssetLoading,
    MenuMain,
    MenuOnline,
    MenuConnect,
    RoundLocal,
    RoundOnline,
    Win,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "images/ggrs_logo.png")]
    pub ggrs_logo: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub default_font: Handle<Font>,
}

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = round::Input;
    type State = u8;
    type Address = PeerId;
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        // asset loading
        .add_loading_state(
            LoadingState::new(AppState::AssetLoading)
                .continue_to_state(AppState::MenuMain)
                .load_collection::<FontAssets>()
                .load_collection::<ImageAssets>(),
        )
        // ggrs plugin
        .add_plugins(GgrsPlugin::<GGRSConfig>::default())
        .set_rollback_schedule_fps(FPS)
        .add_systems(ReadInputs,round::input)
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_reflect::<Velocity>()
        .rollback_component_with_reflect::<Checksum>()
        .rollback_resource_with_reflect::<FrameCount>()
        // rollback schedule
        .add_systems(
            GgrsSchedule,
            (
                apply_inputs,
                update_velocity,
                move_players,
                increase_frame_count,
                checksum_players,
            )
                .chain(),
        )
        // main menu
        .add_systems(OnEnter(AppState::MenuMain), menu::main::setup_ui)
        .add_systems(
            Update,
            (menu::main::btn_visuals, menu::main::btn_listeners)
                .run_if(in_state(AppState::MenuMain)),
        )
        .add_systems(OnExit(AppState::MenuMain), menu::main::cleanup_ui)
        //online menu
        .add_systems(OnEnter(AppState::MenuOnline), menu::online::setup_ui)
        .add_systems(
            Update,
            (
                update_lobby_id,
                update_lobby_id_display,
                update_lobby_btn,
                menu::online::btn_visuals,
                menu::online::btn_listeners,
            )
                .run_if(in_state(AppState::MenuOnline)),
        )
        .add_systems(OnExit(AppState::MenuOnline), menu::online::cleanup_ui)
        // connect menu
        .add_systems(
            OnEnter(AppState::MenuConnect),
            (create_matchbox_socket, menu::connect::setup_ui),
        )
        .add_systems(
            Update,
            (
                update_matchbox_socket,
                menu::connect::btn_visuals,
                menu::connect::btn_listeners,
            )
                .run_if(in_state(AppState::MenuConnect)),
        )
        .add_systems(
            OnExit(AppState::MenuConnect),
            (menu::connect::cleanup, menu::connect::cleanup_ui),
        )
        // win menu
        .add_systems(OnEnter(AppState::Win), menu::win::setup_ui)
        .add_systems(
            Update,
            (menu::win::btn_visuals, menu::win::btn_listeners).run_if(in_state(AppState::Win)),
        )
        .add_systems(OnExit(AppState::Win), menu::win::cleanup_ui)
        // local round
        .add_systems(OnEnter(AppState::RoundLocal), (round::setup_ui, setup_round, spawn_players))
        .add_systems(Update, (check_win, round::btn_visuals, round::btn_listeners).run_if(in_state(AppState::RoundLocal)))
        .add_systems(OnExit(AppState::RoundLocal), (round::cleanup, round::cleanup_ui))
        // online round
        .add_systems(OnEnter(AppState::RoundOnline), (round::setup_ui, setup_round, spawn_players))
        .add_systems(
            Update,
            (check_win, print_p2p_events, round::btn_visuals, round::btn_listeners).run_if(in_state(AppState::RoundOnline)),
        )
        .add_systems(OnExit(AppState::RoundOnline), (round::cleanup, round::cleanup_ui));

    app.run();
}
