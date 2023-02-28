use crate::prelude::*;
use bevy::{log, prelude::*};

use super::{bodies::BodyDefinition, prelude::*, Field};

/// This bundle is used to
#[derive(Bundle, Clone)]
pub struct ToolBundle {
    /// the kind of tool represented by this entity
    tool: Tool,

    /// the body of the tool, most tools at least occupy one grid element, the cutter occupies multiple grid elements
    body: GridBody,

    /// the rendering of the tool is at the moment done via the game logic
    #[bundle]
    sprite: SpriteBundle,
}

type RealBlobFilter = (Without<Tool>, With<RealBlob>);
/// system function that applies the cutter tool and thereby the cutout operation on blobs
pub fn apply_cutter_tool(
    mut commands: Commands,
    tool_query: Query<(&Tool, &GridBody)>,
    mut blob_query: Query<(&mut Blob, &mut GridBody), RealBlobFilter>,
    mut block_query: Query<&mut Block>,
    mut ev_view: EventWriter<ViewUpdate>,
    turn: Res<Turn>,
) {
    if !turn.is_new_turn() {
        return;
    }
    //~

    if let Ok((mut _blob, mut body)) = blob_query.get_single_mut() {
        for (tool, tool_body) in tool_query.iter() {
            if matches!(tool, Tool::Cutter(_)) {
                // 1. get all block positions of cutter on the field
                let tool_positions: Vec<IVec2> = tool_body
                    .blocks
                    .iter()
                    .map(|block_id| block_query.get(*block_id).unwrap().position)
                    .collect();

                // 2. get all tool position that are also occupied by a blob and store the block ids
                let blocks_of_blob: Vec<Entity> = body
                    .blocks
                    .iter()
                    .filter(|block_id| {
                        tool_positions.contains(&block_query.get(**block_id).unwrap().position)
                    })
                    .copied()
                    .collect();

                // 3. if blob blocks and tool blocks have the same len perform cutout
                if blocks_of_blob.len() == tool_positions.len() {
                    // apply the cutout on the blob body
                    body.cutout(
                        &blocks_of_blob,
                        tool_body.pivot,
                        &mut commands,
                        &mut ev_view,
                        &mut block_query,
                    );
                }
            }
        }
    }
}

pub fn apply_movement_tools(
    field_query: Query<&Field>,
    query_tool: Query<&Tool, Without<Blob>>,
    mut query: Query<(Entity, &mut Blob, &mut GridBody)>,
    mut block_query: Query<(Entity, &mut Block)>,
    mut ev_view: EventWriter<ViewUpdate>,
    turn: Res<Turn>,
) {
    if !turn.is_new_turn() {
        return;
    }
    //~

    let field = field_query.single();
    let state = field.get_field_state();

    // apply tool if a tool is applied over the pivot
    query
        .iter_mut()
        .filter(|e| !e.1.cutout && e.1.active)
        .for_each(|(blob_id, mut blob, mut body)| {
            if let Some(element) = state.get_element(body.pivot) {
                if let FieldElementKind::Tool(tool_entity) = element.kind {
                    let tool = query_tool.get(tool_entity).unwrap();
                    match *tool {
                        Tool::Move(d) => {
                            blob.movement = d.into();
                        }
                        Tool::Rotate(d) => {
                            log::info!("Rotation tool at {},{}", body.pivot.x, body.pivot.y);
                            let mut block_iter = block_query.iter_mut().map(|(_, block)| block);
                            match d {
                                RotateDirection::Left => {
                                    body.rotate_left(&mut block_iter, &mut ev_view, blob_id)
                                }
                                RotateDirection::Right => {
                                    body.rotate_right(&mut block_iter, &mut ev_view, blob_id)
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
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

    let id = commands.spawn_empty().id();

    let block_children = match tool {
        Tool::Cutter(tb) => {
            let body_def = BodyDefinition::as_blob(gen_tetris_body(tb));
            let mut vec =
                Block::spawn_blocks_of_blob(commands, &body_def, coordinate, id, field_id, false);
            vec.push(id);
            vec
        }
        _ => vec![id],
    };

    commands
        .entity(id)
        .insert(ToolBundle {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::ONE * PX_PER_TILE - 2.0),
                    ..Default::default()
                },
                transform: Transform::from_translation(position),
                texture: assets.get_tool_image(tool).clone(),
                ..Default::default()
            },
            tool,
            body: GridBody {
                pivot: coordinate,
                blocks: block_children, //todo keep track of blocks
                transferred: false,
            },
        })
        .insert(Block {
            position: coordinate,
            group: Some(id),
            relative_position: Some(IVec2::ZERO),
            field: field_id,
        })
        .insert(Name::new(format!("Tool-{}", tool)));

    commands.entity(field_id).push_children(&[id]);

    id
}

pub fn despawn_tool(commands: &mut Commands, tool_id: Entity, query: &Query<&GridBody>) {
    if let Ok(body) = query.get(tool_id) {
        for block_id in body.blocks.iter() {
            commands.entity(*block_id).despawn_recursive();
        }
    }
    commands.entity(tool_id).despawn_recursive();
}
