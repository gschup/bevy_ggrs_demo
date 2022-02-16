use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ggrs::{Rollback, RollbackIdProvider, SessionType};
use bytemuck::{Pod, Zeroable};
use ggrs::{InputStatus, P2PSession, PlayerHandle};

use crate::{GGRSConfig, NUM_PLAYERS};

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
const MOV_SPEED: f32 = 0.5;
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

#[derive(Default, Reflect, Component)]
pub struct Velocity(Vec2);

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}

pub fn input(_handle: In<PlayerHandle>, keyboard_input: Res<bevy::input::Input<KeyCode>>) -> Input {
    let mut inp: u8 = 0;

    if keyboard_input.pressed(KeyCode::W) {
        inp |= INPUT_UP;
    }
    if keyboard_input.pressed(KeyCode::A) {
        inp |= INPUT_LEFT;
    }
    if keyboard_input.pressed(KeyCode::S) {
        inp |= INPUT_DOWN;
    }
    if keyboard_input.pressed(KeyCode::D) {
        inp |= INPUT_RIGHT;
    }

    Input { inp }
}

pub fn setup_round(mut commands: Commands) {
    commands.insert_resource(FrameCount::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(ARENA_SIZE, ARENA_SIZE)),
            ..Default::default()
        },
        ..Default::default()
    });
}

pub fn spawn_players(mut commands: Commands, mut rip: ResMut<RollbackIdProvider>) {
    let r = ARENA_SIZE / 4.;

    for handle in 0..NUM_PLAYERS {
        let rot = handle as f32 / NUM_PLAYERS as f32 * 2. * std::f32::consts::PI;
        let x = r * rot.cos();
        let y = r * rot.sin();

        let mut transform = Transform::from_translation(Vec3::new(x, y, 1.));
        transform.rotate(Quat::from_rotation_z(rot));

        commands
            .spawn_bundle(SpriteBundle {
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
            .insert(Rollback::new(rip.next_id()));
    }
}

pub fn print_events(mut session: ResMut<P2PSession<GGRSConfig>>) {
    for event in session.events() {
        info!("GGRS Event: {:?}", event);
    }
}

pub fn cleanup_round(query: Query<Entity, With<Player>>, mut commands: Commands) {
    commands.remove_resource::<FrameCount>();
    commands.remove_resource::<P2PSession<GGRSConfig>>();
    commands.remove_resource::<SessionType>();
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
    mut query: Query<(&mut Transform, &mut Velocity, &Player), With<Rollback>>,
    inputs: Res<Vec<(Input, InputStatus)>>,
) {
    for (mut transf, mut v, p) in query.iter_mut() {
        let input = match inputs[p.handle].1 {
            InputStatus::Confirmed => inputs[p.handle].0.inp,
            InputStatus::Predicted => inputs[p.handle].0.inp,
            InputStatus::Disconnected => 0, // disconnected players do nothing
        };

        let steer = if input & INPUT_LEFT != 0 && input & INPUT_RIGHT == 0 {
            1.
        } else if input & INPUT_LEFT == 0 && input & INPUT_RIGHT != 0 {
            -1.
        } else {
            0.
        };

        let accel = if input & INPUT_DOWN != 0 && input & INPUT_UP == 0 {
            -1.
        } else if input & INPUT_DOWN == 0 && input & INPUT_UP != 0 {
            1.
        } else {
            0.
        };

        let vel = &mut v.0;
        let up = transf.up().xy();
        let right = transf.right().xy();

        // car drives forward / backward
        *vel += (accel * MOV_SPEED) * up;

        // rotate car
        let rot = steer * ROT_SPEED * (vel.length() / MAX_SPEED);
        transf.rotate(Quat::from_rotation_z(rot));

        // very realistic tire friction
        let forward_vel = up * vel.dot(up);
        let right_vel = right * vel.dot(right);

        *vel = forward_vel + right_vel * DRIFT;
        if accel.abs() < 0.01 {
            *vel *= FRICTION;
        }

        // constrain velocity
        *vel = vel.clamp_length_max(MAX_SPEED);
    }
}

pub fn move_players(mut query: Query<(&mut Transform, &Velocity), With<Rollback>>) {
    for (mut t, v) in query.iter_mut() {
        // apply velocity
        t.translation.x += v.0.x;
        t.translation.y += v.0.y;

        // constrain cube to plane
        let bounds = (ARENA_SIZE - CUBE_SIZE) * 0.5;
        t.translation.x = t.translation.x.clamp(-bounds, bounds);
        t.translation.y = t.translation.y.clamp(-bounds, bounds);
    }
}
