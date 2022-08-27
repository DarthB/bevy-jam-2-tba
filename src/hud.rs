use bevy::{prelude::*, ui::FocusPolicy};
use player_state::{MoveDirection, PlayerState, RotateDirection, Tool};

use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagImage {
    tool_status: Tool,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagHover {
    tool_status: Tool,
    is_hovered: bool,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagInventory {
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
            focus_policy: FocusPolicy::Pass,
            color: Color::WHITE.into(),
            ..Default::default()
        })
        .insert(Name::new("Toolbar"))
        .with_children(|cb| {
            spawn_tool_button(cb, Tool::Move(MoveDirection::Left), &assets);
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
    .insert(Name::new(format!("Button: {}", tool)))
    .insert(UITagImage { tool_status: tool })
    .with_children(|parent| {
        parent
            .spawn_bundle(
                TextBundle::from_section(
                    "INF",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 16.0,
                        color: Color::GREEN,
                    },
                )
                .with_text_alignment(TextAlignment {
                    vertical: VerticalAlign::Bottom,
                    horizontal: HorizontalAlign::Center,
                }),
            )
            .insert(Name::new(format!("Inventory {}", tool)))
            .insert(UITagInventory { tool_status: tool })
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
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    })
                    .insert(Name::new(format!("Hover {}", tool)))
                    .insert(UITagHover {
                        tool_status: tool,
                        is_hovered: false,
                    });
            });
    });
}

pub fn update_toolbar(
    mut query_images: Query<(&mut UiImage, &mut UITagImage)>,
    mut query_overlay: Query<(&mut UiColor, &mut UITagHover)>,
    player_state: Res<PlayerState>,
    assets: Res<GameAssets>,
) {
    for (mut img, mut tag) in query_images.iter_mut() {
        if let Some(sel_tool) = player_state.selected_tool {
            match (sel_tool, &mut tag.tool_status) {
                (Tool::Move(new_sel), Tool::Move(in_hud)) => *in_hud = new_sel,
                (Tool::Rotate(new_sel), Tool::Rotate(in_hud)) => *in_hud = new_sel,
                (Tool::Cutter(new_sel), Tool::Cutter(in_hud)) => *in_hud = new_sel,
                _ => {}
            }
        }
        *img = assets.get_tool_image(tag.tool_status).clone().into();
    }

    for (mut hover, mut tag) in query_overlay.iter_mut() {
        if let Some(sel_tool) = player_state.selected_tool {
            match (sel_tool, &mut tag.tool_status) {
                (Tool::Move(new_sel), Tool::Move(in_hud)) => *in_hud = new_sel,
                (Tool::Rotate(new_sel), Tool::Rotate(in_hud)) => *in_hud = new_sel,
                (Tool::Cutter(new_sel), Tool::Cutter(in_hud)) => *in_hud = new_sel,
                _ => {}
            }
        }

        let color: UiColor = if let Some(selected_tool) = player_state.selected_tool {
            if selected_tool == tag.tool_status {
                assets.selected_button_color.into()
            } else if tag.is_hovered {
                assets.hover_button_color.into()
            } else {
                assets.normal_button_color.into()
            }
        } else if tag.is_hovered {
            assets.hover_button_color.into()
        } else {
            assets.normal_button_color.into()
        };

        *hover = color;
    }
}

pub fn toolbar_button_system(
    mut interaction_query: Query<(&Interaction, &UITagImage), Changed<Interaction>>,
    mut hover_query: Query<(&mut UiColor, &mut UITagHover)>,
    assets: Res<GameAssets>,
    mut player_state: ResMut<PlayerState>,
    mut turn: ResMut<Turn>,
) {
    for (mut color, mut tag_hover) in &mut hover_query {
        for (interaction, tag) in &mut interaction_query {
            if tag_hover.tool_status != tag.tool_status {
                continue;
            }
            //~

            match *interaction {
                Interaction::Clicked => {
                    *color = assets.clicked_button_color.into();
                    match tag_hover.tool_status {
                        Tool::Play => {
                            turn.pause = false;
                        }
                        Tool::Stop => {
                            turn.pause = true;
                        }
                        _ => {}
                    }
                    player_state.selected_tool = Some(tag_hover.tool_status);
                }
                Interaction::Hovered => {
                    *color = assets.hover_button_color.into();
                    tag_hover.is_hovered = true
                }
                Interaction::None => {
                    *color = assets.normal_button_color.into();
                    tag_hover.is_hovered = false
                }
            }
        }
    }
}
