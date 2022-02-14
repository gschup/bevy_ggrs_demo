use bevy::prelude::*;
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
const MOV_SPEED: f32 = 1.0;
const ROT_SPEED: f32 = 0.05;
const MAX_SPEED: f32 = 5.0;
const FRICTION: f32 = 0.9;
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
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

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
        transform: Transform::from_xyz(0., 0., -1.),
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

        let mut transform = Transform::from_translation(Vec3::new(x, y, 0.));
        transform.rotate(Quat::from_rotation_z(rot));

        commands
            .spawn_bundle(SpriteBundle {
                transform,
                sprite: Sprite {
                    color: PLAYER_COLORS[handle],
                    custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
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
    for (mut t, mut v, p) in query.iter_mut() {
        let input = match inputs[p.handle].1 {
            InputStatus::Confirmed => inputs[p.handle].0.inp,
            InputStatus::Predicted => inputs[p.handle].0.inp,
            InputStatus::Disconnected => INPUT_LEFT, // disconnected players spin
        };

        // rotate left or right
        if input & INPUT_LEFT != 0 && input & INPUT_RIGHT == 0 {
            t.rotate(Quat::from_rotation_z(ROT_SPEED));
        }
        if input & INPUT_LEFT == 0 && input & INPUT_RIGHT != 0 {
            t.rotate(Quat::from_rotation_z(-ROT_SPEED));
        }

        let (_, _, rot_z) = t.rotation.to_euler(EulerRot::XYZ);

        // accelerate forward or backward or slow down if no acceleration is pressed
        if input & INPUT_UP != 0 && input & INPUT_DOWN == 0 {
            v.x += MOV_SPEED * rot_z.cos();
            v.y += MOV_SPEED * rot_z.sin();
        }
        if input & INPUT_UP == 0 && input & INPUT_DOWN != 0 {
            v.x -= MOV_SPEED * rot_z.cos();
            v.y -= MOV_SPEED * rot_z.sin();
        }
        if input & INPUT_UP == 0 && input & INPUT_DOWN == 0 {
            v.x *= FRICTION;
            v.y *= FRICTION;
        }

        // constrain velocity
        let mag = (v.x * v.x + v.y * v.y).sqrt();
        if mag > MAX_SPEED {
            let factor = MAX_SPEED / mag;
            v.x *= factor;
            v.y *= factor;
        }
    }
}

pub fn move_players(mut query: Query<(&mut Transform, &Velocity), With<Rollback>>) {
    for (mut t, v) in query.iter_mut() {
        // apply velocity
        t.translation.x += v.x;
        t.translation.y += v.y;

        // constrain cube to plane
        let bounds = (ARENA_SIZE - CUBE_SIZE) * 0.5;
        t.translation.x = t.translation.x.clamp(-bounds, bounds);
        t.translation.y = t.translation.y.clamp(-bounds, bounds);
    }
}
