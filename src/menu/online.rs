use bevy::prelude::*;

use crate::{AppState, FontAssets, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, TEXT};

use super::connect::ConnectData;

#[derive(Component)]
pub struct MenuOnlineUI;

#[derive(Component)]
pub enum MenuOnlineBtn {
    LobbyMatch,
    QuickMatch,
    Back,
}

pub fn setup_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    // ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(MenuOnlineUI);

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
            // lobby id display
            parent.spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                text: Text::with_section(
                    "Enter a 4-digit ID!",
                    TextStyle {
                        font: font_assets.default_font.clone(),
                        font_size: 32.,
                        color: TEXT,
                    },
                    Default::default(),
                ),
                ..Default::default()
            });

            // lobby match button
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
                            "Lobby Match",
                            TextStyle {
                                font: font_assets.default_font.clone(),
                                font_size: 40.0,
                                color: TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuOnlineBtn::LobbyMatch);

            // quick match button
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
                            "Quick Match",
                            TextStyle {
                                font: font_assets.default_font.clone(),
                                font_size: 40.0,
                                color: TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuOnlineBtn::QuickMatch);

            // back button
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
                                font: font_assets.default_font.clone(),
                                font_size: 40.0,
                                color: TEXT,
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(MenuOnlineBtn::Back);
        })
        .insert(MenuOnlineUI);
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<MenuOnlineBtn>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
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
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<(&Interaction, &MenuOnlineBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
            match btn {
                MenuOnlineBtn::LobbyMatch => {
                    commands.insert_resource(ConnectData {
                        lobby_id: "fighter?next=2".to_owned(),
                    });
                    state
                        .set(AppState::MenuConnect)
                        .expect("Could not change state.");
                }
                MenuOnlineBtn::QuickMatch => {
                    commands.insert_resource(ConnectData {
                        lobby_id: "fighter?next=2".to_owned(),
                    });
                    state
                        .set(AppState::MenuConnect)
                        .expect("Could not change state.");
                }
                MenuOnlineBtn::Back => {
                    state
                        .set(AppState::MenuMain)
                        .expect("Could not change state.");
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<MenuOnlineUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
