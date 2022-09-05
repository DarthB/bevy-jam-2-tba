use crate::prelude::*;
use bevy::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct ToolComponent {
    pub kind: Tool,

    pub relative_positions: Option<Vec<IVec2>>,
}

impl Default for ToolComponent {
    fn default() -> Self {
        Self {
            kind: Tool::Move(MoveDirection::Down),
            relative_positions: Default::default(),
        }
    }
}

/// spawns a tool into the world such that it can affect Blobs on the factory field
pub fn spawn_tool(
    commands: &mut Commands,
    tool: Tool,
    coordinate: IVec2,
    field_id: Entity,
    field: &Field,
    assets: &GameAssets,
) -> Entity {
    let (px, py) = field.coords_to_px(coordinate.x, coordinate.y);
    let position = Vec3::new(px, py, Z_OVERLAY);

    let id = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::ONE * PX_PER_TILE - 2.0),
                ..Default::default()
            },
            transform: Transform::from_translation(position),
            texture: assets.get_tool_image(tool).clone(),
            ..Default::default()
        })
        .insert(ToolComponent {
            kind: tool,
            relative_positions: None,
        })
        .insert(Block {
            position: coordinate,
            blob: None,
            relative_position: None,
            field: field_id,
        })
        .insert(Name::new(format!("Tool-{}", tool)))
        .id();

    commands.entity(field_id).push_children(&[id]);

    id
}
