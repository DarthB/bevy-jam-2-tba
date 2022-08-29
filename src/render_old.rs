use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, prelude::*, render::texture::DEFAULT_IMAGE_HANDLE, log};

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
    fn bounds(&self) -> (IVec2, IVec2);

    fn dimensions(&self) -> (usize, usize) {
        let b = self.bounds();
        (
            (b.1.x - b.0.x) as usize,
            (b.1.y - b.0.y) as usize,
        )
    }

    fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32);

    fn get_render_id(&self, r: i32, c: i32) -> i32;

    fn spawn_normal(&self) -> bool {
        false
    }

    fn spawn_pivot(&self) -> bool {
        false
    }

    fn spawn_origin(&self) -> bool {
        false
    }

    fn spawn_additional_debug(&self) -> bool {
        false
    }

    fn get_sprite_info(&self, num: i32, assets: &GameAssets) -> SpriteInfo;

    fn adapt_render_entities(&self, cb: &mut EntityCommands, r: i32, c: i32);

    fn spawn_render_entities(&self, _id: Entity, cb: &mut ChildBuilder, assets: &GameAssets) {
        if self.spawn_normal() {
            let b = self.bounds();
            for r in b.0.y..b.1.y {
                for c in b.0.x..b.1.x {
                    let num = self.get_render_id(r as i32, c as i32);
                    let info = self.get_sprite_info(num, assets);

                    let dims = self.dimensions();
                    let (x, y) = coords_to_px(c as i32, r as i32, dims.0, dims.1);

                    let mut ec = cb.spawn_bundle(SpriteBundle {
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
                    });
                    ec.insert(Name::new(format!("grid {}:{}", r, c)));
                    self.adapt_render_entities(&mut ec, r as i32, c as i32);
                }
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

        #[cfg(feature = "debug")]
        if self.spawn_additional_debug() {
            cb.spawn_bundle(SpatialBundle::default())
                .with_children(|cb| {
                    let b = self.bounds();
                    for x in b.0.x..b.1.x {
                        for y in b.0.y..b.1.y {
                            if self.get_render_id(y as i32, x as i32) == -1 {
                                continue;
                            }

                            let (px, py) = self.coords_to_px(x as i32, y as i32);
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
                            .insert(DebugOccupiedTag { parent: _id });
                        }
                    }
                })
                .insert(Name::new("Debug occupied"));
        }
    }
}

impl RenderableGrid for Blob {
    fn bounds(&self) -> (IVec2, IVec2) {
        (IVec2::ZERO, IVec2::splat(Blob::size() as i32))
    }

    fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32) {
        coords_to_px(x + self.coordinate.x, y + self.coordinate.y, Blob::size(), Blob::size())
    }

    fn get_render_id(&self, r: i32, c: i32) -> i32 {
        let abs_pos: Vec<IVec2> = self.relative_positions.iter()
            .map(|v| *v + IVec2::new(4,4))
            .collect();
        if abs_pos.contains(&IVec2{x:c, y:r}) {
            1
        } else {
            0
        }
    }

    fn spawn_normal(&self) -> bool {
        false
    }

    fn spawn_pivot(&self) -> bool {
        true
    }

    #[cfg(feature = "debug")]
    fn spawn_origin(&self) -> bool {
        true
    }

    fn get_sprite_info(&self, num: i32, _assets: &GameAssets) -> SpriteInfo {
        let mut reval = if num == 1 {
            SpriteInfo {
                color: Color::default(),
                z: Z_SOLID + 1.0,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            }
        } else {
            SpriteInfo {
                color: Color::rgba(0.5, 0.5, 0.5, 0.25),
                z: Z_TRANS + 1.0,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            }
        };
        reval.image = self.texture.clone();
        reval
    }

    fn adapt_render_entities(&self, cb: &mut EntityCommands, r: i32, c: i32) {
        cb.insert(Coordinate {
            r,
            c,
        });
    }
}

impl RenderableGrid for Target {
    
    fn bounds(&self) -> (IVec2, IVec2) {
        (IVec2::ZERO, IVec2::splat(Target::size() as i32))
    }

    fn coords_to_px(&self, mut x: i32, mut y: i32) -> (f32, f32) {
        if let Some(coord) = &self.coordinate {
            x += coord.c;
            y += coord.r;
        }
        coords_to_px(x, y, Target::size(), Target::size())
    }

    fn get_render_id(&self, r: i32, c: i32) -> i32 {
        self.body[r as usize * Target::size() + c as usize]
    }

    fn spawn_normal(&self) -> bool {
        false
    }

    fn spawn_pivot(&self) -> bool {
        true
    }

    #[cfg(feature = "debug")]
    fn spawn_origin(&self) -> bool {
        true
    }

    fn get_sprite_info(&self, num: i32, _assets: &GameAssets) -> SpriteInfo {
        let mut reval = if num == 1 {
            SpriteInfo {
                color: Color::default(),
                z: Z_SOLID+5.,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            }
        } else {
            SpriteInfo {
                color: Color::rgba(0.5, 0.5, 0.5, 0.25),
                z: Z_TRANS+5.,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            }
        };
        reval.image = self.texture.clone();
        reval
    }

    fn adapt_render_entities(&self, cb: &mut EntityCommands, r: i32, c: i32) {
        cb.insert(Coordinate {
            r,
            c,
        });
    }
}

impl RenderableGrid for Field {
 
    fn bounds(&self) -> (IVec2, IVec2) {
        
        let top = self.allow_overlap.top as i32;
        let left = self.allow_overlap.left as i32;
        let bottom = (self.mov_size().1 + self.allow_overlap.bottom) as i32;
        let right = (self.mov_size().0 + self.allow_overlap.right) as i32;

        (IVec2::new(-left, -top), IVec2::new(right, bottom))
    }

    fn get_sprite_info(&self, num: i32, assets: &GameAssets) -> SpriteInfo {
        match num {
            -1 => SpriteInfo {
                color: Color::BLACK,
                z: Z_SOLID,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            },
            1 => SpriteInfo {
                color: Color::WHITE,
                z: Z_SOLID,
                image: self.brick_image.clone(),
            },
            2 => SpriteInfo {
                image: DEFAULT_IMAGE_HANDLE.typed(),
                color: Color::GRAY,
                z: Z_SOLID,
            },
            3 => SpriteInfo {
                image: DEFAULT_IMAGE_HANDLE.typed(),
                color: Color::NAVY,
                z: Z_SOLID,
            },
            _ => {
                if let Ok(tool) = TryInto::<Tool>::try_into(num) {
                    SpriteInfo {
                        color: Color::WHITE,
                        z: Z_SOLID,
                        image: assets.get_tool_image(tool).clone(),
                    }
                } else {
                    SpriteInfo {
                        color: Color::WHITE,
                        z: Z_SOLID,
                        image: self.movable_area_image.clone(),
                    }
                }
            }
        }
    }

    fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32) {
        let woo = coords_to_px(x, y, self.movable_size.1, self.movable_size.0);
        (woo.0, woo.1)
    }

    fn get_render_id(&self, r: i32, c: i32) -> i32 {
        let b = self.bounds();
        if r < b.0.y || r >= b.1.y || c < b.0.x || c >= b.1.x {
            //log::warn!("Shall not happen! Out of bounds");
            return 3
        }

        let state = self.get_field_state();
        if let Some(element) = state.get_element(IVec2::new(c,r)) {
            match element.kind {
                FieldElementKind::Empty => 0,
                FieldElementKind::OutOfRegion => 2,
                FieldElementKind::Block => 1,
                FieldElementKind::Tool(t) => t.into(),
            }
        } else if r < 0 || r >= self.mov_size().1 as i32 
            || c<0 || c >= self.mov_size().0 as i32 {
            log::warn!("Shall not happen, under mov.size");
            2
        } else {
            log::warn!("Shall not happen, ELSE Branch");
            0
        }
    }
    fn spawn_normal(&self) -> bool {
        true
    }

    fn spawn_pivot(&self) -> bool {
        true
    }

    #[cfg(feature = "debug")]
    fn spawn_origin(&self) -> bool {
        true
    }

    fn spawn_additional_debug(&self) -> bool {
        false
    }

    fn adapt_render_entities(&self, cb: &mut EntityCommands, r: i32, c: i32) {
        cb.insert(Coordinate {
            r,
            c,
        });
        cb.insert(FieldRenderTag {});
    }
}

pub fn grid_update_render_entities<T: Component + RenderableGrid>(
    query_top: Query<(&Children, &T)>,
    mut query: Query<(&mut Sprite, &mut Transform, &mut Handle<Image>, &Coordinate)>,
    assets: Res<GameAssets>,
) {
    for (children, renderable_grid) in query_top.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut t, mut texture, coord)) = query.get_mut(child) {
                let num = renderable_grid.get_render_id(coord.r, coord.c);
                let info = renderable_grid.get_sprite_info(num, &assets);

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
        for (mut sprite, coord) in query_sprite.iter_mut() {
            if coord.c < 0 || coord.r < 0 {
                continue;
            }
            //~

            let field_state = field.get_field_state();
            sprite.color = if let Some(element) = field_state.get_element(IVec2::new(coord.c, coord.r)) {
                match element.kind {
                    FieldElementKind::Block => Color::RED,
                    _ => Color::WHITE,
                }
            } else {
                Color::WHITE
            }
        }
    }
}
