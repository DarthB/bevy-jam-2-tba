use std::fmt::Debug;

use bevy::{ecs::system::EntityCommands, log, prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};

use itertools::Itertools;

use crate::{game_assets::GameAssets, prelude::*};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct Field {
    pub movable_area_color: Color,

    pub edge_color: Color,

    pub movable_size: (usize, usize),

    pub additional_grids: UiRect<usize>,

    pub allow_overlap: UiRect<usize>,

    pub occupied: Vec<i32>,

    pub tracks_occupied: bool,

    pub remove_full_lines: bool,

    pub movable_area_image: Handle<Image>,

    pub brick_image: Handle<Image>,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct FactoryFieldTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct ProductionFieldTag {}

pub struct FieldDeltaEvent {
    pub conditional_y: i32,

    pub delta_y: i32,

    pub field_id: Entity,
}

type FieldMutator = dyn Fn(&mut Field, (i32, i32), usize);

impl Field {
    pub fn as_factory(assets: &GameAssets) -> Self {
        Field {
            movable_area_color: Color::MIDNIGHT_BLUE,
            edge_color: Color::rgb(0.0, 0.2, 0.5),
            movable_size: (10, 18),
            additional_grids: UiRect {
                left: 4,
                right: 4,
                top: 0,
                bottom: 0,
            },
            allow_overlap: UiRect {
                left: 10,
                right: 0,
                top: 4,
                bottom: 9,
            },
            movable_area_image: assets.block_factory_floor.clone(),
            brick_image: assets.block_blob.clone(),
            ..Default::default()
        }
    }

    pub fn as_production_field(assets: &GameAssets) -> Self {
        let mut reval = Field {
            edge_color: Color::rgba(0.25, 0.0, 0.0, 1.0),
            allow_overlap: UiRect {
                left: 0,
                right: 0,
                top: 10,
                bottom: 0,
            },
            tracks_occupied: true,
            movable_area_image: assets.block_tetris_floor.clone(),
            brick_image: assets.block_blob.clone(),
            ..Default::default()
        };

        let num = reval.movable_size.0 * reval.movable_size.1;
        reval.occupied = vec![0; num];

        reval
    }

    pub fn tracks_occupied(&self) -> bool {
        self.tracks_occupied
    }

    pub fn occupied(&self) -> &Vec<i32> {
        &self.occupied
    }

    pub fn is_idx_valid(&self, idx: usize) -> bool {
        idx < self.occupied.len()
    }

    pub fn max_idx(&self) -> usize {
        self.occupied.len() - 1
    }

    pub fn mutate_at_coordinates(&mut self, coords: &Vec<(i32, i32)>, mutator: &FieldMutator) {
        for &(x, y) in coords {
            if x < 0 || y < 0 {
                continue;
            }

            let idx = self.coords_to_idx(x as usize, y as usize);
            if self.is_idx_valid(idx) {
                mutator(self, (x, y), idx);
            } else {
                log::warn!(
                    "Tried to mutate something at idx {idx}, although {} is the maximum idx for this field.",
                    self.max_idx()
                );
            }
        }
    }

    pub fn occupy_coordinates(&mut self, coords: &Vec<(i32, i32)>) {
        self.mutate_at_coordinates(coords, &|field, _, idx| {
            field.occupied[idx] = 1;
        });
    }

    pub fn deoccupy_coordinates(&mut self, coords: &Vec<(i32, i32)>) {
        self.mutate_at_coordinates(coords, &|field, _, idx| {
            field.occupied[idx] = 0;
        });
    }

    pub fn any_coordinate_occupied(&self, coords: &Vec<(i32, i32)>) -> (bool, Option<(i32, i32)>) {
        for (x, y) in coords {
            let res = self.is_coordinate_occupied(*x, *y);
            if res.0 {
                return res;
            }
        }
        (false, None)
    }

    /// return true if the given coordinate can be occupied, otherwise provide
    /// an information how the movement can be reduced such that the border is reached
    pub fn is_coordinate_occupied(&self, x: i32, y: i32) -> (bool, Option<(i32, i32)>) {
        // first check if the space is already occupied:
        if self.tracks_occupied && x >= 0 && y >= 0 {
            let idx = self.coords_to_idx(x as usize, y as usize);
            if idx < self.occupied.len() && self.occupied[idx] > 0 {
                return (true, None);
            }
        }

        let mut x_correct = 0;
        let mut y_correct = 0;

        let border = self.coordinate_limits();

        let left_check = x < border.left;
        if left_check {
            x_correct = border.left - x;
        }

        let right_check = x >= border.right;
        if right_check {
            x_correct = border.right - x;
        }

        let up_check = y < border.top;
        if up_check {
            y_correct = border.top - y;
        }

        let down_check = y >= border.bottom;
        if down_check {
            y_correct = border.bottom - y;
        }

        let occupied_by_edge = left_check || right_check || up_check || down_check;

        if x_correct != 0 || y_correct != 0 {
            (occupied_by_edge, Some((x + x_correct, y + y_correct)))
        } else {
            (occupied_by_edge, None)
        }
    }

    pub fn coords_to_idx(&self, x: usize, y: usize) -> usize {
        x + y * self.movable_size.0
    }

    pub fn size(&self) -> (usize, usize) {
        (
            self.movable_size.0 + self.additional_grids.left + self.additional_grids.right,
            self.movable_size.1 + self.additional_grids.top + self.additional_grids.bottom,
        )
    }

    pub fn mov_size(&self) -> (usize, usize) {
        self.movable_size
    }

    /// Calculates the offset in pixels used to render to the movable area, if uneven edges are used.
    pub fn offset(&self) -> (f32, f32) {
        let f = PX_PER_TILE / 2.0;
        (
            self.additional_grids.right as f32 * f - self.additional_grids.left as f32 * f,
            self.additional_grids.top as f32 * f - self.additional_grids.bottom as f32 * f,
        )
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

    pub fn full_lines(&self) -> Vec<usize> {
        let mut reval: Vec<usize> = Vec::new();
        if !self.tracks_occupied || !self.remove_full_lines {
            return reval;
        }
        //~

        for r in 0..self.movable_size.1 {
            let mut full_line = true;
            for c in 0..self.movable_size.0 {
                full_line = full_line && self.occupied()[self.coords_to_idx(c, r)] > 0
            }
            if full_line {
                reval.push(r);
            }
        }

        reval
    }

    pub fn deoccupy_lines(&mut self, lines: &Vec<usize>) {
        if !self.tracks_occupied {
            return;
        }

        for r in lines {
            for c in 0..self.movable_size.0 {
                let idx = self.coords_to_idx(c, *r);
                if idx < self.occupied.len() {
                    self.occupied[idx] = 0;
                }
            }
        }
    }

    pub fn move_down_if_possible(&mut self) {
        for r in (0..self.mov_size().1 - 1).rev() {
            for c in 0..self.mov_size().0 {
                let idx_up = self.coords_to_idx(c, r);
                let idx_below = self.coords_to_idx(c, r + 1);
                if self.occupied[idx_up] != 0 && self.occupied[idx_below] == 0 {
                    self.occupied[idx_below] = self.occupied[idx_up];
                    self.occupied[idx_up] = 0;
                }
            }
        }
    }
}

impl Default for Field {
    fn default() -> Self {
        Self {
            movable_area_color: Color::GRAY,
            edge_color: Color::DARK_GRAY,
            movable_size: (10, 18),
            additional_grids: UiRect {
                left: 1,
                right: 1,
                top: 0,
                bottom: 1,
            },
            allow_overlap: UiRect {
                left: 0,
                right: 0,
                top: 5,
                bottom: 0,
            },
            occupied: Vec::new(),
            tracks_occupied: false,
            remove_full_lines: true,
            movable_area_image: DEFAULT_IMAGE_HANDLE.typed(),
            brick_image: DEFAULT_IMAGE_HANDLE.typed(),
        }
    }
}

pub fn spawn_field(
    commands: &mut Commands,
    field: Field,
    name: &str,
    trans: Vec3,
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
        field.spawn_render_entities(id, cb);
    })
    .insert(Name::new(name.to_string()))
    .insert(field);
    adapter(&mut ec);
    ec.id()
}

pub fn remove_field_lines(mut query_field: Query<&mut Field>) {
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
}
