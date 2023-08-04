use bevy::prelude::*;

use crate::{AppState, FontAssets, BUTTON_TEXT, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

#[derive(Component)]
pub struct WinUI;

#[derive(Component)]
pub enum MenuWinBtn {
    Back,
}

#[derive(Resource)]
pub struct MatchData {
    pub result: String,
}

pub fn setup_ui(mut commands: Commands, match_data: Res<MatchData>, font_assets: Res<FontAssets>) {
    // ui camera
    commands.spawn(Camera2dBundle::default()).insert(WinUI);

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // match result string
            parent.spawn(TextBundle::from_section(
                match_data.result.clone(),
                TextStyle {
                    font: font_assets.default_font.clone(),
                    font_size: 96.,
                    color: BUTTON_TEXT,
                },
            ));
            // back to menu button
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
                .insert(MenuWinBtn::Back);
        })
        .insert(WinUI);

    commands.remove_resource::<MatchData>();
}

pub fn btn_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuWinBtn>),
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
    mut interaction_query: Query<(&Interaction, &MenuWinBtn), Changed<Interaction>>,
) {
    for (interaction, btn) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            match btn {
                MenuWinBtn::Back => {
                    state.set(AppState::MenuMain);
                }
            }
        }
    }
}

pub fn cleanup_ui(query: Query<Entity, With<WinUI>>, mut commands: Commands) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}
