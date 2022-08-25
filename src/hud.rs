use bevy::prelude::*;
use player_state::{MoveDirection, PlayerState, RotateDirection, Tool};

use crate::{bodies::TetrisBricks, game_assets::GameAssets, player_state, PX_PER_ICON};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagImage {
    tool_status: Tool,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagHover {
    tool_status: Tool,
}

pub fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>) {
    let ysize = PX_PER_ICON * 5.0 + 4.0 * 8.0;

    commands
        .spawn_bundle(NodeBundle {
            node: Node {
                size: Vec2::new(PX_PER_ICON, ysize),
            },
            style: Style {
                size: Size::new(Val::Px(PX_PER_ICON), Val::Px(ysize)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                // Reverse means from top to bottom
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            color: Color::WHITE.into(),
            ..Default::default()
        })
        .with_children(|cb| {
            spawn_tool_button(cb, Tool::Direction(MoveDirection::Left), &assets);
            spawn_tool_button(cb, Tool::Rotate(RotateDirection::Left), &assets);
            spawn_tool_button(cb, Tool::Cutter(TetrisBricks::Square), &assets);
            spawn_tool_button(cb, Tool::Play, &assets);
            spawn_tool_button(cb, Tool::Stop, &assets)
        });
}

fn spawn_tool_button(cb: &mut ChildBuilder, tool: player_state::Tool, assets: &GameAssets) {
    cb.spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(PX_PER_ICON), Val::Px(PX_PER_ICON)),
            // center button
            margin: UiRect::all(Val::Auto),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        image: assets.get_tool_image(tool).clone().into(),
        ..default()
    })
    .insert(UITagImage { tool_status: tool })
    .with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(PX_PER_ICON), Val::Px(PX_PER_ICON)),
                    // center button
                    margin: UiRect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: assets.normal_button_color.into(),
                ..default()
            })
            .insert(UITagHover { tool_status: tool });
    });
}

pub fn update_toolbar(
    mut query_images: Query<(&mut UiImage, &UITagImage)>,
    mut query_overlay: Query<(&mut UiColor, &UITagHover)>,
    player_state: Res<PlayerState>,
    assets: Res<GameAssets>,
) {
    for (mut img, tag) in query_images.iter_mut() {
        *img = assets.get_tool_image(tag.tool_status).clone().into();
    }

    for (mut hover, _tag) in query_overlay.iter_mut() {
        let color: UiColor = if let Some(selected_tool) = player_state.selected_tool {
            if selected_tool == _tag.tool_status {
                assets.selected_button_color.into()
            } else {
                assets.normal_button_color.into()
            }
        } else {
            assets.normal_button_color.into()
        };

        *hover = color;
    }
}
