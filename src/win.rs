use bevy::prelude::*;

use crate::{AppState, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, TEXT};

#[derive(Component)]
pub struct WinUI;

#[derive(Component)]
pub struct ContBtn;

pub struct MatchData {
    pub result: String,
}

pub fn setup_win_ui(
    mut commands: Commands,
    match_data: Res<MatchData>,
    asset_server: Res<AssetServer>,
) {
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(WinUI);

    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect::all(Val::Px(0.)),
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            // match result string
            parent.spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                text: Text::with_section(
                    match_data.result.clone(),
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 96.,
                        color: TEXT,
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
            // back to menu button
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: Rect::all(Val::Px(16.)),
                        padding: Rect::all(Val::Px(16.)),
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Back to Menu",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(ContBtn);
        })
        .insert(WinUI);

    commands.remove_resource::<MatchData>();
}

pub fn update_cont_btn(
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<ContBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(AppState::Menu).expect("Could not change state.");
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

pub fn cleanup_win_ui(query: Query<Entity, With<WinUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
