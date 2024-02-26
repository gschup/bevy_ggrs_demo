use bevy::utils::HashMap;
use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ggrs::ggrs::InputStatus;
use bevy_ggrs::{AddRollbackCommandExtension, GgrsTime, LocalInputs, LocalPlayers, PlayerInputs, Rollback, Session};
use bytemuck::{Pod, Zeroable};

use crate::{FontAssets, BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::{
    checksum::Checksum,
    menu::win::MatchData,
    AppState, GGRSConfig, NUM_PLAYERS,
};

const INPUT_UP: u8 = 0b0001;
const INPUT_DOWN: u8 = 0b0010;
const INPUT_LEFT: u8 = 0b0100;
const INPUT_RIGHT: u8 = 0b1000;

const BLUE: Color = Color::rgb(0.8, 0.6, 0.2);
const ORANGE: Color = Color::rgb(0., 0.35, 0.8);
const MAGENTA: Color = Color::rgb(0.9, 0.2, 0.2);
const GREEN: Color = Color::rgb(0.35, 0.7, 0.35);
const PLAYER_COLORS: [Color; 4] = [BLUE, ORANGE, MAGENTA, GREEN];

const PLAYER_SIZE: f32 = 50.;
const MOV_SPEED: f32 = 0.1;
const ROT_SPEED: f32 = 0.05;
const MAX_SPEED: f32 = 7.5;
const FRICTION: f32 = 0.98;
const DRIFT: f32 = 0.95;
const ARENA_SIZE: f32 = 720.0;
const CUBE_SIZE: f32 = 0.2;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct Input {
    pub inp: u8,
}

#[derive(Default, Component)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component)]
pub struct RoundUI;

#[derive(Component)]
pub enum RoundBtn {
    Back,
}

#[derive(Component)]
pub struct RoundEntity;

#[derive(Default, Reflect, Component)]
pub struct Velocity(pub Vec2);

#[derive(Default, Reflect, Component)]
pub struct CarControls {
    accel: f32,
    steer: f32,
}

#[derive(Default, Reflect, Hash, Resource)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}

pub fn input(mut commands: Commands, local_players: Res<LocalPlayers>, keyboard_input: Res<bevy::prelude::Input<KeyCode>>) {
    let local_players = &local_players.0;
    let mut local_inputs = HashMap::new();

    for handle in local_players.iter() {
        let local = local_players.len() > 1;
        let mut input = 0;

        if !local || *handle == 0 {
            if keyboard_input.pressed(KeyCode::W) {
                input |= INPUT_UP;
            }
            if keyboard_input.pressed(KeyCode::A) {
                input |= INPUT_LEFT;
            }
            if keyboard_input.pressed(KeyCode::S) {
                input |= INPUT_DOWN;
            }
            if keyboard_input.pressed(KeyCode::D) {
                input |= INPUT_RIGHT;
            }
        } else {
            if keyboard_input.pressed(KeyCode::Up) {
                input |= INPUT_UP;
            }
            if keyboard_input.pressed(KeyCode::Left) {
                input |= INPUT_LEFT;
            }
            if keyboard_input.pressed(KeyCode::Down) {
                input |= INPUT_DOWN;
            }
            if keyboard_input.pressed(KeyCode::Right) {
                input |= INPUT_RIGHT;
            }
        }

        local_inputs.insert(*handle, Input {inp: input });
    }

    commands.insert_resource(LocalInputs::<GGRSConfig>(local_inputs));
}

pub fn setup_round(mut commands: Commands) {
    println!("OH YEAH");
    commands.insert_resource(FrameCount::default());
    commands
        .spawn(Camera2dBundle::default())
        .insert(RoundEntity);
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(0., 0., 0.),
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(ARENA_SIZE, ARENA_SIZE)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RoundEntity);
}

pub fn spawn_players(mut commands: Commands) {
    let r = ARENA_SIZE / 4.;

    for handle in 0..NUM_PLAYERS {
        let rot = handle as f32 / NUM_PLAYERS as f32 * 2. * std::f32::consts::PI;
        let x = r * rot.cos();
        let y = r * rot.sin();

        let mut transform = Transform::from_translation(Vec3::new(x, y, 1.));
        transform.rotate(Quat::from_rotation_z(rot));

        commands
            .spawn(SpriteBundle {
                transform,
                sprite: Sprite {
                    color: PLAYER_COLORS[handle],
                    custom_size: Some(Vec2::new(PLAYER_SIZE * 0.5, PLAYER_SIZE)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Player { handle })
            .insert(Velocity::default())
            .insert(CarControls::default())
            .insert(Checksum::default())
            .add_rollback()
            .insert(RoundEntity);
    }
}

pub fn print_p2p_events(mut session: ResMut<Session<GGRSConfig>>) {
    if let Session::P2P(s) = session.as_mut() {
        for event in s.events() {
            info!("GGRS Event: {:?}", event);
        }
    }
}

pub fn check_win(mut next_state: ResMut<NextState<AppState>>, mut commands: Commands) {
    let condition = false;
    let confirmed = false;

    if condition && confirmed {
        next_state.set(AppState::Win);
        commands.insert_resource(MatchData {
            result: "Orange won!".to_owned(),
        });
    }
}

pub fn cleanup(query: Query<Entity, With<RoundEntity>>, mut commands: Commands) {
    commands.remove_resource::<FrameCount>();
    commands.remove_resource::<LocalPlayers>();
    commands.remove_resource::<Session<GGRSConfig>>();

    // https://github.com/gschup/bevy_ggrs/issues/93 
    commands.insert_resource(Time::new_with(GgrsTime::default()));

    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/*
 * UI
 */

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
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
                .insert(RoundBtn::Back);
        })
        .insert(RoundUI);
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RoundBtn>),
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
    mut interaction_query: Query<(&Interaction, &RoundBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                RoundBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<RoundUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/*
 * ROLLBACK SYSTEMS
 */

pub fn increase_frame_count(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

pub fn apply_inputs(
    mut query: Query<(&mut CarControls, &Player)>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    for (mut c, p) in query.iter_mut() {
        let input = match inputs[p.handle].1 {
            InputStatus::Confirmed => inputs[p.handle].0.inp,
            InputStatus::Predicted => inputs[p.handle].0.inp,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        };

        c.steer = if input & INPUT_LEFT != 0 && input & INPUT_RIGHT == 0 {
            1.
        } else if input & INPUT_LEFT == 0 && input & INPUT_RIGHT != 0 {
            -1.
        } else {
            0.
        };

        c.accel = if input & INPUT_DOWN != 0 && input & INPUT_UP == 0 {
            -1.
        } else if input & INPUT_DOWN == 0 && input & INPUT_UP != 0 {
            1.
        } else {
            0.
        };
    }
}

pub fn update_velocity(mut query: Query<(&Transform, &mut Velocity, &CarControls)>) {
    for (t, mut v, c) in query.iter_mut() {
        let vel = &mut v.0;
        let up = t.up().xy();
        let right = t.right().xy();

        // car drives forward / backward
        *vel += (c.accel * MOV_SPEED) * up;

        // very realistic tire friction
        let forward_vel = up * vel.dot(up);
        let right_vel = right * vel.dot(right);

        *vel = forward_vel + right_vel * DRIFT;
        if c.accel.abs() <= 0.0 {
            *vel *= FRICTION;
        }

        // constrain velocity
        *vel = vel.clamp_length_max(MAX_SPEED);
    }
}

pub fn move_players(mut query: Query<(&mut Transform, &Velocity, &CarControls), With<Rollback>>) {
    for (mut t, v, c) in query.iter_mut() {
        let vel = &v.0;
        let up = t.up().xy();

        // rotate car
        let rot_factor = (vel.length() / MAX_SPEED).clamp(0.0, 1.0); // cannot rotate while standing still
        let rot = if vel.dot(up) >= 0.0 {
            c.steer * ROT_SPEED * rot_factor
        } else {
            // negate rotation while driving backwards
            c.steer * ROT_SPEED * rot_factor * -1.0
        };
        t.rotate(Quat::from_rotation_z(rot));

        // apply velocity
        t.translation.x += vel.x;
        t.translation.y += vel.y;

        // constrain cube to plane
        let bounds = (ARENA_SIZE - CUBE_SIZE) * 0.5;
        t.translation.x = t.translation.x.clamp(-bounds, bounds);
        t.translation.y = t.translation.y.clamp(-bounds, bounds);
    }
}
