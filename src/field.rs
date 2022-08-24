use std::fmt::Debug;

use bevy::{ecs::system::EntityCommands, log, prelude::*};

use itertools::Itertools;

use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct Field {
    movable_area_color: Color,

    edge_color: Color,

    movable_size: (usize, usize),

    additional_grids: UiRect<usize>,

    allow_overlap: UiRect<usize>,

    occupied: Vec<bool>,

    tracks_occupied: bool,

    remove_full_lines: bool,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct DebugOccupiedTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct FactoryFieldTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct ProductionFieldTag {}

impl Field {
    pub fn as_factory() -> Self {
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
            ..Default::default()
        }
    }

    pub fn as_production_field() -> Self {
        let mut reval = Field {
            edge_color: Color::rgba(0.25, 0.0, 0.0, 1.0),
            tracks_occupied: true,
            allow_overlap: UiRect {
                left: 0,
                right: 0,
                top: 10,
                bottom: 0,
            },
            ..Default::default()
        };

        let num = reval.movable_size.0 * reval.movable_size.1;
        reval.occupied = vec![false; num];

        reval
    }

    pub fn tracks_occupied(&self) -> bool {
        self.tracks_occupied
    }

    pub fn occupied(&self) -> &Vec<bool> {
        &self.occupied
    }

    pub fn occupy_coordinates(&mut self, coords: &Vec<(i32, i32)>) {
        for (x, y) in coords {
            if *x < 0 || *y < 0 {
                continue;
            }

            let idx = self.coords_to_idx(*x as usize, *y as usize);
            if idx < self.occupied.len() {
                self.occupied[idx] = true;
            } else {
                log::warn!(
                    "Tried to occupy coordinate at idx {idx}, also only {} elemnts exist",
                    self.occupied.len()
                );
            }
        }
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
            if idx < self.occupied.len() && self.occupied[idx] {
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

    pub fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32) {
        let woo = coords_to_px(x, y, self.movable_size.1, self.movable_size.0);
        //let (ox, oy) = self.offset();
        (woo.0, woo.1)
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
                full_line = full_line && self.occupied()[self.coords_to_idx(c, r)]
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
                    self.occupied[idx] = false;
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
        }
    }
}

pub fn spawn_field(
    commands: &mut Commands,
    movable_tile_tex: &Handle<Image>,
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
    ec.with_children(|cb| {
        let (sx, sy) = field.size();
        let (ox, oy) = field.offset();

        cb.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: field.edge_color,
                custom_size: Some(Vec2::new(PX_PER_TILE * sx as f32, PX_PER_TILE * sy as f32)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(ox, oy, Z_FIELD),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Background Sprite"));

        cb.spawn_bundle(SpatialBundle::default())
            .insert(Name::new("Movable Area Group"))
            .with_children(|cb| {
                for x in 0..field.movable_size.0 {
                    for y in 0..field.movable_size.1 {
                        cb.spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(
                                    PX_PER_TILE,
                                    PX_PER_TILE,
                                )),
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: Vec3::new(
                                    (x as f32 - field.mov_size().0 as f32 / 2.0 + 0.5) * PX_PER_TILE + ox, 
                                    (y as f32 - field.mov_size().1 as f32 / 2.0 + 0.5) * PX_PER_TILE + oy,
                                    Z_FIELD + 1.0),
                                ..Default::default()
                            },
                            texture: movable_tile_tex.clone(),
                            ..Default::default()
                        })
                        .insert(Name::new(format!("Sprite {x},{y}")));
                    }
                }
            })
        ;

        #[cfg(feature = "debug")]
        cb.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, Z_OVERLAY),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Pivot Sprite"));

        #[cfg(feature = "debug")]
        {
            let (x, y) = field.coords_to_px(0, 0);
            cb.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(x, y, Z_OVERLAY),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Name::new("Zero Sprite"));
        }

        #[cfg(feature = "debug")]
        if field.tracks_occupied {
            cb.spawn_bundle(SpatialBundle::default())
                .with_children(|cb| {
                    for x in 0..field.mov_size().0 {
                        for y in 0..field.mov_size().1 {
                            let (px, py) = field.coords_to_px(x as i32, y as i32);
                            cb.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::WHITE,
                                    custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: Vec3::new(px, py - PX_PER_TILE / 4.0, Z_OVERLAY),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Name::new(format!("{x}x{y} occupied helper")))
                            .insert(Coordinate {
                                c: x as i32,
                                r: y as i32,
                            })
                            .insert(DebugOccupiedTag {});
                        }
                    }
                })
                .insert(Name::new("Debug occupied"));
        }
    })
    .insert(Name::new(name.to_string()))
    .insert(field);
    adapter(&mut ec);
    ec.id()
}

pub fn update_field_debug(
    query: Query<&Field>,
    mut query_sprite: Query<(&mut Sprite, &Coordinate), With<DebugOccupiedTag>>,
) {
    for field in query.iter() {
        if !field.tracks_occupied() {
            continue;
        }
        //~

        for (mut sprite, coord) in query_sprite.iter_mut() {
            if coord.c < 0 || coord.r < 0 {
                continue;
            }
            //~

            let idx = field.coords_to_idx(coord.c as usize, coord.r as usize);

            sprite.color = if idx < field.occupied().len() && field.occupied()[idx] {
                Color::RED
            } else {
                Color::WHITE
            }
        }
    }
}

pub fn remove_field_lines(
    mut query_field: Query<(&mut Field, &Children)>,
    mut query_blob: Query<&mut Blob>,
    mut turn: ResMut<Turn>,
) {
    for (mut field, children) in query_field.iter_mut() {
        let fl = field.full_lines();
        if fl.is_empty() {
            continue;
        }
        //~

        turn.pause = true;
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

        for child in children {
            // @todo send an event that these rows will are triggerd for destroy and
            // work on this event on blob side instead doing everything here (easier anim later)
            if let Ok(mut blob) = query_blob.get_mut(*child) {
                if blob.coordinate.is_none() {
                    continue;
                }
                let (c, r) = (
                    blob.coordinate.as_ref().unwrap().c - pivot_coord().0 as i32, 
                    blob.coordinate.as_ref().unwrap().r - pivot_coord().1 as i32,
                );
                //~

                let mut blob_coords = blob.occupied_coordinates();
                println!("Blob:{:?}", blob_coords);
                let adapted_blob_coords: Vec<(i32, i32)> = blob_coords
                    .iter()
                    .map(|(x, y)| {(
                        x-pivot_coord().0 as i32,
                        y-pivot_coord().1 as i32
                    )})
                    .collect();
                let filtered_blob_coords: Vec<(i32, i32)> = adapted_blob_coords.iter()
                    .filter(|&&(x, y)| coordinates.contains(&(x, y)))
                    .map(|&(x,y)| (x,y))
                    .collect();

                for (x,y) in filtered_blob_coords {
                    if x >= 0 && y >= 0 {
                        let idx = Blob::coords_to_idx(y as usize - r as usize, x as usize - c as usize);
                        if idx < blob.body.len() {
                            let prior = blob.body[idx];
                            log::info!("Delete sprite at {x},{y} with {idx}, was={prior}");
                            blob.body[idx] = 0;
                        } else {
                            log::warn!(
                                "Cannot delete sprite at {x},{y} with {idx}, as len={}",
                                blob.body.len()
                            );
                        }
                    }
                }
            }
        }

        field.deoccupy_lines(&fl);
    }
}
