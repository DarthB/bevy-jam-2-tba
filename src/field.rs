use std::fmt::Debug;

use bevy::{ecs::system::EntityCommands, prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};
use itertools::Itertools;

use crate::{game_assets::GameAssets, prelude::*};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone)]
pub struct Field {
    pub movable_size: (usize, usize),

    pub overlap_left: u32,
    pub overlap_right: u32,
    pub overlap_top: u32,
    pub overlap_bottom: u32,

    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub movable_area_image: Handle<Image>,

    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub brick_image: Handle<Image>,

    #[cfg_attr(feature = "debug", inspectable(ignore))]
    field_state: FieldState,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct FactoryFieldTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct ProductionFieldTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct FieldRenderTag {}

pub type FieldMutator = dyn Fn(&mut Field, (i32, i32), usize);

impl Field {
    pub fn as_factory(assets: &GameAssets) -> Self {
        Field {
            movable_size: (10, 24),
            overlap_left: 4,
            overlap_right: 4,
            overlap_top: 4,
            overlap_bottom: 0,
            movable_area_image: assets.block_factory_floor.clone(),
            brick_image: assets.block_blob.clone(),
            ..Default::default()
        }
    }

    pub fn as_production_field(assets: &GameAssets) -> Self {
        Field {
            overlap_left: 0,
            overlap_right: 0,
            overlap_top: 10,
            overlap_bottom: 0,

            movable_area_image: assets.block_tetris_floor.clone(),
            brick_image: assets.block_blob.clone(),
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

            movable_area_image: DEFAULT_IMAGE_HANDLE.typed(),
            brick_image: DEFAULT_IMAGE_HANDLE.typed(),
            field_state: FieldState::default(),
        }
    }
}

pub fn generate_field_states(
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
    adapter: &dyn Fn(&mut EntityCommands),
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
    adapter(&mut ec);

    id
}
