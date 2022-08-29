use std::fmt::Debug;

use bevy::{ecs::system::EntityCommands, log, prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};

use crate::{game_assets::GameAssets, prelude::*};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone)]
pub struct Field {
    pub movable_size: (usize, usize),

    pub allow_overlap: UiRect<usize>,

    pub movable_area_image: Handle<Image>,

    pub brick_image: Handle<Image>,

    //#[cfg_attr(feature = "debug", inspectable(ignore))]
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
        let reval = Field {
            movable_size: (10, 18),
            allow_overlap: UiRect {
                left: 4,
                right: 4,
                top: 4,
                bottom: 9,
            },
            movable_area_image: assets.block_factory_floor.clone(),
            brick_image: assets.block_blob.clone(),
            ..Default::default()
        };

        reval
    }

    pub fn as_production_field(assets: &GameAssets) -> Self {
        let reval = Field {
            allow_overlap: UiRect {
                left: 0,
                right: 0,
                top: 10,
                bottom: 0,
            },
            movable_area_image: assets.block_tetris_floor.clone(),
            brick_image: assets.block_blob.clone(),
            ..Default::default()
        };

        reval
    }

    pub fn bounds(&self) -> (IVec2, IVec2) {
        (
            IVec2::new(
                -(self.allow_overlap.top as i32), 
                -(self.allow_overlap.left as i32),
            ),
            IVec2::new(
                (self.mov_size().0 + self.allow_overlap.right) as i32,
                (self.mov_size().1 + self.allow_overlap.bottom) as i32,
            )
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
        block_iter: impl Iterator<Item = (Entity, &'a Block, Option<&'a ToolComponent>)>,
    ) -> &FieldState {
        self.field_state = FieldState::new(self.bounds());

        for x in self.bounds().0.x..self.bounds().1.x {
            for y in self.bounds().0.y..self.bounds().1.y {
                if x < 0 || y < 0 || x >= self.mov_size().0 as i32 || y >= self.mov_size().1 as i32 {
                    let pos = IVec2::new(x,y);
                    self.field_state.set_element(pos, FieldElement { 
                        entity: None, 
                        blob: None,
                        kind: FieldElementKind::OutOfRegion,
                        position: pos 
                    });
                }
            }
        }

        for (entity, block, opt_tool) in block_iter {
            let new_el = if let Some(blob) = block.blob {
                FieldElement {
                    entity: Some(entity),
                    blob: Some(blob),
                    kind: FieldElementKind::Block,
                    position: block.position,
                }
            } else if let Some(tool) = opt_tool {
                FieldElement {
                    entity: Some(entity),
                    blob: None,
                    kind: FieldElementKind::Tool(tool.kind),
                    position: block.position,
                }
            } else if let Some(old_el) = self.field_state.get_element(block.position) {
                old_el
            } else {
                FieldElement::default()
            };        
            self.field_state.set_element(block.position, new_el);
        }
        &self.field_state
    }

    pub fn mov_size(&self) -> (usize, usize) {
        self.movable_size
    }

    /// returns a rect that gives defines the limits of the corrdinates by respecting
    /// the allow_overlap property
    pub fn coordinate_limits(&self) -> UiRect<i32> {
        UiRect {
            left: -(self.allow_overlap.left as i32),
            right: (self.movable_size.0 + self.allow_overlap.right) as i32,
            top: -(self.allow_overlap.top as i32),
            bottom: (self.movable_size.1 + self.allow_overlap.bottom) as i32,
        }
    }

    /*
    pub fn full_lines(&self) -> Vec<usize> {
        let mut reval: Vec<usize> = Vec::new();
        if !self.tracks_occupied || !self.remove_full_lines {
            return reval;
        }
        //~

        for r in 0..self.movable_size.1 {
            let mut full_line = true;
            for c in 0..self.movable_size.0 {
                full_line = full_line && self.occupied[self.coords_to_idx(c, r).unwrap()] > 0
            }
            if full_line {
                reval.push(r);
            }
        }

        reval
    }
    */

    pub fn deoccupy_lines(&mut self, lines: &Vec<usize>) {
        unimplemented!()
        /*
        if !self.tracks_occupied {
            return;
        }

        for r in lines {
            for c in 0..self.movable_size.0 {
                if let Some(idx) = self.coords_to_idx(c, *r) {
                    self.occupied[idx] = 0;
                }
            }
        }
        */
    }

    pub fn move_down_if_possible(&mut self) {
        unimplemented!()
        /*
        for r in (0..self.mov_size().1 - 1).rev() {
            for c in 0..self.mov_size().0 {
                let idx_up = self.coords_to_idx(c, r).unwrap();
                let idx_below = self.coords_to_idx(c, r + 1).unwrap();
                if self.occupied[idx_up] > 0
                    && self.occupied[idx_up] < 100
                    && self.occupied[idx_below] == 0
                {
                    self.occupied[idx_below] = self.occupied[idx_up];
                    self.occupied[idx_up] = 0;
                }
            }
        }
        */
    }

    pub fn remove_all_tools(&mut self) -> Vec<Tool> {
        let mut reval = Vec::new();
        /*
        
        for r in 0..self.mov_size().1 {
            for c in 0..self.mov_size().0 {
                let idx = self.coords_to_idx(c, r).unwrap();
                let tool: Result<Tool, _> = TryFrom::<i32>::try_from(self.occupied[idx]);
                if let Ok(tool) = tool {
                    reval.push(tool);
                    self.occupied[idx] = 0;
                }
            }
        }
        */

        reval
    }
}

impl Default for Field {
    fn default() -> Self {
        Self {
            movable_size: (10, 18),
            allow_overlap: UiRect {
                left: 0,
                right: 0,
                top: 5,
                bottom: 0,
            },
            movable_area_image: DEFAULT_IMAGE_HANDLE.typed(),
            brick_image: DEFAULT_IMAGE_HANDLE.typed(),
            field_state: FieldState::default()
        }
    }
}

pub fn generate_field_states(
    query_state: Query<(Entity, &mut Block, Option<&ToolComponent>)>,
    mut query_field: Query<(Entity, &mut Field)>,
) {
    for (field_id, mut field) in query_field.iter_mut() {
        let iter = query_state.iter().filter(|(_, block, _)| {
            block.field.is_some() && block.field.unwrap() == field_id
        });
        field.generate_field_state(iter);
    }
}

pub fn spawn_field(
    commands: &mut Commands,
    assets: &GameAssets,
    field: Field,
    name: &str,
    trans: Vec3,
    use_old_rendering: bool,
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let mut ec = commands.spawn_bundle(SpatialBundle {
        transform: Transform {
            translation: trans,
            ..Default::default()
        },
        ..Default::default()
    });
    let id = ec.id();
    ec.with_children(|cb| {
        if use_old_rendering {
            field.spawn_render_entities(id, cb, assets);
        } else {
            // no info from view yet
        }
    })
    .insert(Name::new(name.to_string()))
    .insert(field);
    adapter(&mut ec);
    ec.id()
}

pub fn remove_field_lines(mut query_field: Query<&mut Field>) {
    /*
    for mut field in query_field.iter_mut() {
        let fl: Vec<usize> = field.full_lines();
        if fl.is_empty() {
            continue;
        }
        //~
        log::info!("Removing {} lines", fl.len());

        let coordinates: Vec<(i32, i32)> = fl
            .iter()
            .cartesian_product(0..field.mov_size().0)
            .map(|(y, x)| (x as i32, *y as i32))
            .collect();
        log::info!(
            "Cartesian Product contains  {} elments\n{:?}",
            coordinates.len(),
            coordinates
        );

        field.deoccupy_lines(&fl);
    }
    */
}
