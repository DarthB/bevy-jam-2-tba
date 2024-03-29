//! The field module represents the root element of the entity object tree which via the [`Field`] struct.
//!
//! On the field the [`Target`] marks the shape that has to be filled by the player. Beside that [`Tool`]s and
//! [`Blob`]s which consists of [`Block`]s may inhabit a field.
//!
//! The ingame representation and the data that describes a puzzle is separated. To design a level for Disastris
//! have a look at the module [`crate::data`] and there speicifially the [`crate::data::level::Level`] structure.

use std::fmt::Debug;

use bevy::{log, prelude::*};
use itertools::Itertools;

pub mod blob;
pub mod field_element;
pub mod target;
pub mod tool;

pub mod prelude {
    pub use super::blob::move_blob_by_input;
    pub use super::blob::Blob;
    pub use super::blob::GridBody;

    pub use super::target::Coordinate;
    pub use super::target::Target;

    pub use super::tool::Tool;

    pub use super::Block;
    pub use super::Field;

    pub use super::field_element::FieldElement;
    pub use super::field_element::FieldElementKind;
    pub use super::field_element::FieldState;
}
use self::{prelude::*, tool::despawn_tool};
use crate::{data::prelude::*, render_old::RenderableGrid};

//----------------------------------------------------------------------
// Field Component, Tags and implementation

#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct Field {
    pub movable_size: (usize, usize),

    pub overlap_left: u32,
    pub overlap_right: u32,
    pub overlap_top: u32,
    pub overlap_bottom: u32,

    field_state: FieldState,
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct FieldRenderTag {}

pub type FieldMutator = dyn Fn(&mut Field, (i32, i32), usize);

impl Field {
    pub fn as_factory() -> Self {
        Field {
            movable_size: (10, 24),
            overlap_left: 4,
            overlap_right: 4,
            overlap_top: 4,
            overlap_bottom: 0,
            ..Default::default()
        }
    }

    pub fn as_production_field() -> Self {
        Field {
            overlap_left: 0,
            overlap_right: 0,
            overlap_top: 10,
            overlap_bottom: 0,
            ..Default::default()
        }
    }

    pub fn bounds(&self) -> (IVec2, IVec2) {
        (
            IVec2::new(-(self.overlap_left as i32), -(self.overlap_top as i32)),
            IVec2::new(
                self.mov_size().0 as i32 + self.overlap_right as i32,
                self.mov_size().1 as i32 + self.overlap_bottom as i32,
            ),
        )
    }

    pub fn get_field_state(&self) -> &FieldState {
        &self.field_state
    }

    /// This method can be called by systems to update the field state cache
    /// This ensures that at some places where field state is queried a lot it is not regenerated
    /// all the time (once per system should be fine)
    pub fn generate_field_state<'a>(
        &mut self,
        block_iter: impl Iterator<Item = (Entity, &'a Block)>,
        tool_query: &Query<&Tool>,
        blob_query: &Query<&Blob>,
        target_query: &Query<&Target>,
    ) -> &FieldState {
        let old_fieldstate = self.field_state.clone();
        self.field_state = FieldState::new(self.bounds());

        let target = target_query.single();

        for x in self.bounds().0.x..self.bounds().1.x {
            for y in self.bounds().0.y..self.bounds().1.y {
                let pos = IVec2::new(x, y);

                if x < 0 || y < 0 || x >= self.mov_size().0 as i32 || y >= self.mov_size().1 as i32
                {
                    // elements that are out of the actual playing field and are used for visual overlaps
                    // over the playfield border
                    self.field_state.set_element(
                        pos,
                        FieldElement {
                            is_target: false,
                            entity: None,
                            kind: FieldElementKind::OutOfMovableRegion,
                            position: pos,
                        },
                    );
                } else {
                    // elements that are part of the actual playing field
                    let is_target = target.occupied_coordinates().contains(&(x, y));
                    if is_target {
                        self.field_state.set_element(
                            pos,
                            FieldElement {
                                is_target: true,
                                entity: None,
                                kind: FieldElementKind::Empty,
                                position: pos,
                            },
                        );
                    }
                }
            }
        }

        for (entity, block) in block_iter {
            let is_target = old_fieldstate
                .get_element(block.position)
                .as_ref()
                .map_or(false, |e| e.is_target);

            let new_el = if let Some(group) = block.group {
                // 1. case group of blob
                if let Ok(_blob) = blob_query.get(group) {
                    FieldElement {
                        is_target,
                        entity: Some(entity),
                        kind: FieldElementKind::Block(Some(group)),
                        position: block.position,
                    }
                // 2. cae group of a tool
                } else if let Ok(_tool) = tool_query.get(group) {
                    FieldElement {
                        is_target,
                        entity: Some(entity),
                        kind: FieldElementKind::Tool(group),
                        position: block.position,
                    }
                } else {
                    // may happen in one frame after the cutout in this case we just return the former element
                    // at this position and wait for the update
                    old_fieldstate.get_element(block.position).unwrap()
                }
            } else {
                FieldElement {
                    is_target,
                    entity: Some(entity),
                    kind: FieldElementKind::Block(None),
                    position: block.position,
                }
            };

            self.field_state.set_element(block.position, new_el);
        }
        &self.field_state
    }

    pub fn mov_size(&self) -> (usize, usize) {
        self.movable_size
    }

    pub fn remove_all_tools(
        &mut self,
        commands: &mut Commands,
        query: &Query<&Tool, With<GridBody>>,
        query_body: &Query<&GridBody>,
    ) -> Vec<Tool> {
        let state = self.get_field_state();

        let buffer: Vec<(Entity, Tool)> = state
            .into_iter()
            .filter(|f| matches!(f.kind, FieldElementKind::Tool(_)))
            .map(|e| match e.kind {
                FieldElementKind::Tool(tool_entity) => {
                    if let Ok(t) = query.get(tool_entity) {
                        (tool_entity, *t)
                    } else {
                        panic!("Shall not happen");
                    }
                }
                _ => panic!("Shall not happen because of previous filter."),
            })
            .unique_by(|&(id, _)| id)
            .collect();

        for &(id, _) in buffer.iter() {
            despawn_tool(commands, id, query_body);
        }

        buffer.iter().map(|&(_, t)| t).collect()
    }
}

impl Default for Field {
    fn default() -> Self {
        Self {
            movable_size: (10, 18),
            overlap_left: 0,
            overlap_right: 0,
            overlap_top: 5,
            overlap_bottom: 0,

            field_state: FieldState::default(),
        }
    }
}

pub fn field_states_generation_system(
    query_state: Query<(Entity, &mut Block)>,
    query_blob: Query<&Blob>,
    query_tool: Query<&Tool>,
    query_target: Query<&Target>,
    mut query_field: Query<(Entity, &mut Field)>,
) {
    for (field_id, mut field) in query_field.iter_mut() {
        let iter = query_state
            .iter()
            .filter(|(_, block)| block.field == field_id);
        //log::info!("Blocks on Field {:?} = {}", field_id, iter.count());
        field.generate_field_state(iter, &query_tool, &query_blob, &query_target);
    }
}

pub fn spawn_field(
    commands: &mut Commands,
    assets: &GameAssets,
    field: Field,
    name: &str,
    trans: Vec3,
) -> Entity {
    let mut ec = commands.spawn(SpatialBundle {
        transform: Transform {
            translation: trans,
            ..Default::default()
        },
        ..Default::default()
    });

    let id = ec.id();

    ec.with_children(|cb| {
        field.spawn_render_entities(id, cb, assets);
    })
    .insert(Name::new(name.to_string()))
    .insert(field);

    id
}

//----------------------------------------------------------------------
// Block component and implementation

/// is used to mark that rect in a grid that shall be used as a pivot
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct PivotTag {}

/// is used to mark the 0,0 coordinate in grid based coordinate systems
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct OriginTag {}

/// Reperesents a block that occupies a coordinate in the field
///
/// In a parent/child tree of entities the block entity marks a leaf node. It
/// can either be part of:
///
/// * A [`self::Blob`] that is a collection of blocks that form shapes like the tetris stones
/// * A [`self::Tool`] that is spawned on the filed and mutates the behavior of [`self::Blob`]s that touch it.
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct Block {
    /// the position of the block in the coordinate system of the field
    pub position: IVec2,

    /// the relative position of the block in respect to the parent grid body (if parent is [`self::Blob`])
    pub relative_position: Option<IVec2>,

    /// the group management entity of this block, (e.g. [`self::Blob`] or [`self::Tool`]), if any
    pub group: Option<Entity>,

    /// Reference to the parent field of this block
    pub field: Entity,
}

impl Block {
    pub fn spawn_blocks_of_blob(
        commands: &mut Commands,
        body_def: &BodyDefinition,
        pivot: IVec2,
        group_id: Entity,
        field: Entity,
        handle_zero_position: bool,
    ) -> Vec<Entity> {
        let mut reval = vec![];
        for v in body_def.get_relative_positions() {
            if v == IVec2::ZERO && !handle_zero_position {
                continue;
            }
            //~

            let mut ec = commands.spawn_empty();
            if v == IVec2::ZERO {
                ec.insert(PivotTag {});
            }
            let id = ec
                .insert(Block {
                    position: pivot + v,
                    group: Some(group_id),
                    relative_position: Some(v),
                    field,
                })
                .insert(Name::new(format!("Block {},{}", v.x, v.y)))
                .id();

            log::info!("Spawn block {:?} at {v} with field {:?}", id, field);
            reval.push(id);
        }
        reval
    }
}
