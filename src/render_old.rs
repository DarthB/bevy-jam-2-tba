use crate::prelude::*;
use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};

pub struct SpriteInfo {
    pub image: Handle<Image>,
    pub color: Color,
    pub z: f32,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct DebugOccupiedTag {
    pub parent: Entity,
}

pub trait RenderableGrid {
    fn rows(&self) -> usize;

    fn cols(&self) -> usize;

    fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32);

    fn get_render_id(&self, r: usize, c: usize) -> i32;

    fn spawn_pivot(&self) -> bool {
        false
    }

    fn spawn_origin(&self) -> bool {
        false
    }

    fn spawn_additional_debug(&self) -> bool {
        false
    }

    fn get_sprite_info(&self, num: i32) -> SpriteInfo;

    fn spawn_render_entities(&self, id: Entity, cb: &mut ChildBuilder) {
        for r in 0..self.rows() {
            for c in 0..self.cols() {
                let num = self.get_render_id(r, c);
                let info = self.get_sprite_info(num);

                let (x, y) = coords_to_px(c as i32, r as i32, self.rows(), self.cols());

                cb.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: info.color,
                        custom_size: Some(Vec2::ONE * PX_PER_TILE - 2.0),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(x, y, info.z),
                        ..Default::default()
                    },
                    texture: info.image,
                    ..Default::default()
                })
                .insert(Coordinate {
                    r: r as i32,
                    c: c as i32,
                })
                .insert(Name::new(format!("grid {}:{}", r, c)));
            }
        }

        if self.spawn_pivot() {
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
            .insert(Name::new("Pivot Point"));
        }

        if self.spawn_origin() {
            let (x, y) = self.coords_to_px(0, 0);
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
            .insert(Name::new("Origin Point"));
        }

        if self.spawn_additional_debug() {
            cb.spawn_bundle(SpatialBundle::default())
                .with_children(|cb| {
                    for x in 0..self.cols() {
                        for y in 0..self.rows() {
                            if self.get_render_id(y, x) == -1 {
                                continue;
                            }

                            // @hack as the edge is not handled here
                            // update in special system (another hack) --> update_field_debug
                            let x = x - 1;
                            //let y = y + 1;

                            let (px, py) = self.coords_to_px(x as i32, y as i32);
                            cb.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::WHITE,
                                    custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: Vec3::new(px, py + PX_PER_TILE / 4.0, Z_OVERLAY),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Name::new(format!("{x}x{y} occupied helper")))
                            .insert(Coordinate {
                                c: x as i32,
                                r: y as i32,
                            })
                            .insert(DebugOccupiedTag { parent: id });
                        }
                    }
                })
                .insert(Name::new("Debug occupied"));
        }
    }
}

impl RenderableGrid for Blob {
    fn rows(&self) -> usize {
        Blob::size()
    }

    fn cols(&self) -> usize {
        Blob::size()
    }

    fn coords_to_px(&self, mut x: i32, mut y: i32) -> (f32, f32) {
        if let Some(coord) = &self.coordinate {
            x += coord.c;
            y += coord.r;
        }
        coords_to_px(x, y, Blob::size(), Blob::size())
    }

    fn get_sprite_info(&self, num: i32) -> SpriteInfo {
        let mut reval = if num == 1 {
            SpriteInfo {
                color: Color::default(),
                z: Z_SOLID,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            }
        } else {
            SpriteInfo {
                color: Color::rgba(0.5, 0.5, 0.5, 0.25),
                z: Z_TRANS,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            }
        };
        reval.image = self.texture.clone();
        reval
    }

    #[cfg(feature = "debug")]
    fn spawn_origin(&self) -> bool {
        true
    }

    fn spawn_pivot(&self) -> bool {
        true
    }

    fn get_render_id(&self, r: usize, c: usize) -> i32 {
        self.body[r * Blob::size() + c]
    }
}

impl RenderableGrid for Field {
    fn rows(&self) -> usize {
        self.mov_size().1 + self.additional_grids.top + self.additional_grids.bottom
    }

    fn cols(&self) -> usize {
        self.mov_size().0 + self.additional_grids.left + self.additional_grids.right
    }

    fn get_sprite_info(&self, num: i32) -> SpriteInfo {
        match num {
            -1 => SpriteInfo {
                color: self.edge_color,
                z: Z_SOLID,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            },
            1 => SpriteInfo {
                color: Color::WHITE,
                z: Z_SOLID,
                image: self.brick_image.clone(),
            },
            _ => SpriteInfo {
                color: Color::WHITE,
                z: Z_SOLID,
                image: self.movable_area_image.clone(),
            },
        }
    }

    fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32) {
        let woo = coords_to_px(x, y, self.movable_size.1, self.movable_size.0);
        //let (ox, oy) = self.offset();
        (woo.0, woo.1)
    }

    fn get_render_id(&self, r: usize, c: usize) -> i32 {
        if r < self.additional_grids.top
            || r >= self.additional_grids.top + self.mov_size().1
            || c < self.additional_grids.left
            || c >= self.additional_grids.left + self.mov_size().0
        {
            -1
        } else if self.tracks_occupied {
            let r = r - self.additional_grids.top;
            let c = c - self.additional_grids.left;
            let idx = self.coords_to_idx(c, r);
            if idx < self.occupied.len() {
                self.occupied[idx]
            } else {
                0
            }
        } else {
            0
        }
    }

    fn spawn_pivot(&self) -> bool {
        true
    }

    #[cfg(feature = "debug")]
    fn spawn_origin(&self) -> bool {
        true
    }

    fn spawn_additional_debug(&self) -> bool {
        self.tracks_occupied
    }
}

pub fn grid_update_render_entities<T: Component + RenderableGrid>(
    query_top: Query<(&Children, &T)>,
    mut query: Query<(&mut Sprite, &mut Transform, &mut Handle<Image>, &Coordinate)>,
) {
    for (children, renderable_grid) in query_top.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut t, mut texture, coord)) = query.get_mut(child) {
                let num = renderable_grid.get_render_id(coord.r as usize, coord.c as usize);
                let info = renderable_grid.get_sprite_info(num);

                sprite.color = info.color;
                *texture = info.image;
                t.translation.z = info.z;
            }
        }
    }
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

            sprite.color = if idx < field.occupied().len() && field.occupied()[idx] > 0 {
                Color::RED
            } else {
                Color::WHITE
            }
        }
    }
}
