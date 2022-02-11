use bevy::prelude::*;

use crate::AppState;

pub struct MenuData {
    quick_match_btn: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // quick match button
    let quick_match_btn = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),            // center button
                justify_content: JustifyContent::Center, // horizontally center child text
                align_items: AlignItems::Center,         // vertically center child text
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
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();
    commands.insert_resource(MenuData { quick_match_btn });
}

pub fn menu(
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state
                    .set(AppState::Connect)
                    .expect("Could not change state.");
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

pub fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands
        .entity(menu_data.quick_match_btn)
        .despawn_recursive();
}
