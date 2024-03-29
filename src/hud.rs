//! Contains all the HUD (human user display) related components. The spawn_* methods are used to initialize
//! the HUD. The update_* methods keep the HUD visuals in-sync with the player state. To identify what has to be
//! done when the HUD is clicked the UITag* components are used.

use crate::{DisastrisAppState, PX_PER_ICON};
use bevy::text::Text2dBounds;
use bevy::{prelude::*, ui::FocusPolicy};

use crate::field::prelude::*;
use crate::movement::prelude::*;
use crate::prelude::*;

pub mod prelude {
    pub use super::UITagHover;
    pub use super::UITagImage;
    pub use super::UITagInventory;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagImage {
    tool_status: Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagHover {
    tool_status: Tool,
    is_hovered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component)]
pub struct UITagInventory {
    tool_status: Tool,
}

pub fn spawn_hud(mut commands: Commands, assets: Res<GameAssets>) {
    let ysize = PX_PER_ICON * 7.0 + 4.0 * 8.0;

    let pos = Vec2::new(240., 0.);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(PX_PER_ICON),
                height: Val::Px(ysize),
                left: Val::Px(pos.x),
                top: Val::Px(pos.y),
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
            background_color: Color::WHITE.into(),
            ..Default::default()
        })
        .insert(Name::new("Toolbar"))
        .with_children(|cb| {
            spawn_tool_button(cb, Tool::Simulate, &assets);
            spawn_tool_button(cb, Tool::Move(MoveDirection::default()), &assets);
            spawn_tool_button(cb, Tool::Rotate(RotateDirection::default()), &assets);
            spawn_tool_button(cb, Tool::Cutter(TetrisBricks::default()), &assets);
            spawn_tool_button(cb, Tool::Eraser, &assets);
            spawn_tool_button(cb, Tool::EraseAll, &assets);
            spawn_tool_button(cb, Tool::Reset, &assets);
        });
}

fn spawn_tool_button(cb: &mut ChildBuilder, tool: Tool, assets: &GameAssets) {
    cb.spawn(ButtonBundle {
        style: Style {
            width: Val::Px(PX_PER_ICON),
            height: Val::Px(PX_PER_ICON),
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
            .spawn(
                TextBundle::from_section(
                    "INF",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 24.0,
                        color: Color::GREEN,
                    },
                )
                .with_text_alignment(TextAlignment::Center),
            )
            .insert(Name::new(format!("Inventory {}", tool)))
            .insert(UITagInventory { tool_status: tool })
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(PX_PER_ICON),
                            height: Val::Px(PX_PER_ICON),
                            // center button
                            margin: UiRect::all(Val::Auto),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: assets.normal_button_color.into(),
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

pub fn toolbar_images_system(
    mut query_images: Query<(&mut UiImage, &mut UITagImage)>,
    player_state: Res<PlayerStateLevel>,
    assets: Res<GameAssets>,
) {
    for (mut img, mut tag) in query_images.iter_mut() {
        // Ensure tool variant in tag is the same as selected by the player
        if let Some(sel_tool) = player_state.selected_tool {
            match (sel_tool, &mut tag.tool_status) {
                (Tool::Move(new_sel), Tool::Move(in_hud)) => *in_hud = new_sel,
                (Tool::Rotate(new_sel), Tool::Rotate(in_hud)) => *in_hud = new_sel,
                (Tool::Cutter(new_sel), Tool::Cutter(in_hud)) => *in_hud = new_sel,
                _ => {}
            }
        }

        // Ensure its the right image
        *img = assets.get_tool_image(tag.tool_status).clone().into();
    }
}

pub fn toolbar_inventory_system(
    mut query_text: Query<(&mut Text, &UITagInventory)>,
    player_state: Res<PlayerStateLevel>,
) {
    for (mut text, tag) in query_text.iter_mut() {
        if let Some(inv_num) = player_state.num_in_inventory(tag.tool_status) {
            text.sections[0].value = inv_num.to_string();
        }
    }
}

pub fn toolbar_overlays_system(
    mut query_overlay: Query<(&mut BackgroundColor, &mut UITagHover)>,
    player_state: Res<PlayerStateLevel>,
    assets: Res<GameAssets>,
) {
    for (mut hover, mut tag) in query_overlay.iter_mut() {
        // Ensure tool variant in tag is the same as selected by the player
        if let Some(sel_tool) = player_state.selected_tool {
            match (sel_tool, &mut tag.tool_status) {
                (Tool::Move(new_sel), Tool::Move(in_hud)) => *in_hud = new_sel,
                (Tool::Rotate(new_sel), Tool::Rotate(in_hud)) => *in_hud = new_sel,
                (Tool::Cutter(new_sel), Tool::Cutter(in_hud)) => *in_hud = new_sel,
                _ => {}
            }
        }

        // we use the number of tools in the player inventory but for simulation/reset/etc we use usize::MAX
        let num_inv = player_state
            .num_in_inventory(tag.tool_status)
            .unwrap_or(usize::MAX);

        // Select the overlay color
        let color: BackgroundColor = if let Some(selected_tool) = player_state.selected_tool {
            if selected_tool == tag.tool_status {
                if num_inv == 0 {
                    assets.selected_but_unavailable_button_color.into()
                } else {
                    assets.selected_button_color.into()
                }
            } else if num_inv == 0 {
                assets.unavailable_button_color.into()
            } else if tag.is_hovered {
                assets.hover_button_color.into()
            } else {
                assets.normal_button_color.into()
            }
        } else if tag.is_hovered && num_inv != 0 {
            assets.hover_button_color.into()
        } else if num_inv == 0 {
            assets.unavailable_button_color.into()
        } else {
            assets.normal_button_color.into()
        };

        *hover = color;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn toolbar_button_system(
    mut commands: Commands,
    mut field_query: Query<&mut Field>,
    mut interaction_query: Query<(&Interaction, &UITagImage), Changed<Interaction>>,
    query_tool: Query<&Tool, With<GridBody>>,
    query_body: Query<&GridBody>,
    mut hover_query: Query<(&mut BackgroundColor, &mut UITagHover)>,
    assets: Res<GameAssets>,
    mut player_state: ResMut<PlayerStateLevel>,
    mut turn: ResMut<GameStateLevel>,
    mut next_state: ResMut<NextState<DisastrisAppState>>,
) {
    for (mut color, mut tag_hover) in &mut hover_query {
        for (interaction, tag) in &mut interaction_query {
            if tag_hover.tool_status != tag.tool_status {
                continue;
            }
            //~

            match *interaction {
                Interaction::Pressed => {
                    *color = assets.clicked_button_color.into();
                    match tag_hover.tool_status {
                        Tool::Simulate => {
                            turn.simulation_running = false;
                        }
                        Tool::Reset => {
                            // From Placeholder --> PlayLevel
                            next_state.set(DisastrisAppState::Placeholder);
                        }
                        Tool::EraseAll => {
                            if let Ok(mut field) = field_query.get_single_mut() {
                                let tools =
                                    field.remove_all_tools(&mut commands, &query_tool, &query_body);
                                for tool in tools {
                                    player_state.add_to_inventory(tool, 1);
                                }
                            }
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

pub fn spawn_text(
    commands: &mut Commands,
    assets: &GameAssets,
    text: &str,
    pos: Vec2,
    size: Vec2,
    fg_color: Color,
    bg_color: Color,
) {
    let txt_style = TextStyle {
        font: assets.font.clone(),
        font_size: 36.0,
        color: fg_color,
    };

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: bg_color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(0.)),
            ..default()
        })
        .with_children(|cb| {
            cb.spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(text, txt_style)],
                    alignment: TextAlignment::Center,
                    linebreak_behavior: bevy::text::BreakLineOn::WordBoundary,
                },
                text_2d_bounds: Text2dBounds { size },
                transform: Transform::from_translation(Vec3::Z),
                ..default()
            });
        });
}
