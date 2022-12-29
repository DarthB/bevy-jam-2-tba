use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};

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

pub fn coords_to_px(x: i32, y: i32, rs: usize, cs: usize) -> (f32, f32) {
    (
        ((cs as f32 / -2.0) + x as f32) * PX_PER_TILE + PX_PER_TILE / 2.0,
        ((rs as f32 / 2.0) - y as f32) * PX_PER_TILE - PX_PER_TILE / 2.0,
    )
}

pub trait RenderableGrid {
    fn bounds(&self) -> (IVec2, IVec2);

    fn dimensions(&self) -> (usize, usize) {
        let b = self.bounds();
        ((b.1.x - b.0.x) as usize, (b.1.y - b.0.y) as usize)
    }

    fn pivot(&self) -> (i32, i32) {
        (
            (self.dimensions().0 as i32 + self.bounds().0.x) / 2,
            (self.dimensions().1 as i32 + self.bounds().0.y) / 2,
        )
    }

    fn coords_to_px(&self, x: i32, y: i32) -> (f32, f32);

    fn get_render_id(&self, r: i32, c: i32, tool_query: Option<&Query<&Tool>>) -> i32;

    fn spawn_pivot(&self) -> bool {
        true
    }

    fn spawn_origin(&self) -> bool {
        true
    }

    fn spawn_additional_debug(&self) -> bool {
        false
    }

    fn get_sprite_info(&self, num: i32, assets: &GameAssets) -> SpriteInfo;

    fn adapt_render_entities(&self, cb: &mut EntityCommands, r: i32, c: i32);

    fn spawn_render_entities(&self, _id: Entity, cb: &mut ChildBuilder, assets: &GameAssets) {
        #[cfg(feature = "debug")]
        cb.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0., 0.0, Z_OVERLAY),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("NOTRANSLATION"));

        cb.spawn(SpatialBundle::default())
            .insert(Name::new("Sprites"))
            .with_children(|cb| {
                let b = self.bounds();
                for r in b.0.y..b.1.y {
                    for c in b.0.x..b.1.x {
                        let num = self.get_render_id(r as i32, c as i32, None);
                        let info = self.get_sprite_info(num, assets);

                        let (x, y) = self.coords_to_px(c, r);

                        let mut ec = cb.spawn(SpriteBundle {
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
                        ec.insert(FieldRenderTag {});
                        ec.insert(Name::new(format!("grid {}:{}", r, c)));
                        self.adapt_render_entities(&mut ec, r as i32, c as i32);

                        if self.spawn_origin() && r == 0 && c == 0 {
                            cb.spawn(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::GREEN,
                                    custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: Vec3::new(x, y - PX_PER_TILE / 4.0, Z_OVERLAY),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Coordinate { r, c })
                            .insert(OriginTag {})
                            .insert(Name::new("Origin"));
                        }

                        let (pc, pr) = self.pivot();
                        if r == pr as i32 && c == pc as i32 && self.spawn_pivot() {
                            cb.spawn(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::RED,
                                    custom_size: Some(Vec2::ONE * PX_PER_TILE / 4.0),
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: Vec3::new(x, y, Z_OVERLAY),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Coordinate { r, c })
                            .insert(PivotTag {})
                            .insert(Name::new("Pivot Point"));
                        }
                    }
                }
            });

        #[cfg(feature = "debug")]
        if self.spawn_additional_debug() {
            cb.spawn_bundle(SpatialBundle::default())
                .with_children(|cb| {
                    let b = self.bounds();
                    for x in b.0.x..b.1.x {
                        for y in b.0.y..b.1.y {
                            if self.get_render_id(y as i32, x as i32, None) == -1 {
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

impl RenderableGrid for Field {
    fn bounds(&self) -> (IVec2, IVec2) {
        let top = self.overlap_top as i32;
        let left = self.overlap_left as i32;
        let bottom = self.mov_size().1 as i32 + self.overlap_bottom as i32;
        let right = self.mov_size().0 as i32 + self.overlap_right as i32;

        (IVec2::new(-left, -top), IVec2::new(right, bottom))
    }

    fn get_sprite_info(&self, num: i32, assets: &GameAssets) -> SpriteInfo {
        match num {
            -1 => SpriteInfo {
                color: Color::BLACK,
                z: Z_FIELD,
                image: DEFAULT_IMAGE_HANDLE.typed(),
            },
            1 => SpriteInfo {
                color: Color::WHITE,
                z: Z_FIELD,
                image: self.brick_image.clone(),
            },
            2 => SpriteInfo {
                image: DEFAULT_IMAGE_HANDLE.typed(),
                color: Color::GRAY,
                z: Z_FIELD,
            },
            3 => SpriteInfo {
                image: DEFAULT_IMAGE_HANDLE.typed(),
                color: Color::NAVY,
                z: Z_FIELD,
            },
            _ => {
                if let Ok(tool) = TryInto::<Tool>::try_into(num) {
                    SpriteInfo {
                        color: Color::WHITE,
                        z: Z_FIELD,
                        image: assets.get_tool_image(tool).clone(),
                    }
                } else {
                    SpriteInfo {
                        color: Color::WHITE,
                        z: Z_FIELD,
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

    fn get_render_id(&self, r: i32, c: i32, tool_query: Option<&Query<&Tool>>) -> i32 {
        let b = self.bounds();
        if r < b.0.y || r >= b.1.y || c < b.0.x || c >= b.1.x {
            panic!("Shall not happen! r and c for get_render_id out of bounds");
        }

        let state = self.get_field_state();
        if let Some(element) = state.get_element(IVec2::new(c, r)) {
            match element.kind {
                FieldElementKind::Empty => 0,
                FieldElementKind::OutOfMovableRegion => 2,
                FieldElementKind::OutOfValidRegion => -1,
                FieldElementKind::Block(_) => 0, // 2 turns this off somehow as it is the same as empty
                FieldElementKind::Tool(tool_entity) => {
                    let query = tool_query.expect("tool query shall be given");
                    let tool = query.get(tool_entity).expect("tool shall also be there");
                    (*tool).into()
                }
            }
        } else if r < 0 || r >= self.mov_size().1 as i32 || c < 0 || c >= self.mov_size().0 as i32 {
            // render the outside grid in gray
            2
        } else {
            // no field element kind found (e.g. during spawning)
            0
        }
    }

    fn spawn_pivot(&self) -> bool {
        true
    }

    fn spawn_origin(&self) -> bool {
        true
    }

    fn spawn_additional_debug(&self) -> bool {
        false
    }

    fn adapt_render_entities(&self, cb: &mut EntityCommands, r: i32, c: i32) {
        cb.insert(Coordinate { r, c });
    }
}

type RenderGridTuple<'w> = (
    &'w mut Sprite,
    &'w mut Transform,
    &'w mut Handle<Image>,
    &'w Coordinate,
    Option<&'w PivotTag>,
);

pub fn grid_update_render_entities<T: Component + RenderableGrid>(
    query_top: Query<(&Children, &T)>,
    query_1st_children_layer: Query<(&Children, &Name)>,
    query_tool: Query<&Tool>,
    mut query: Query<RenderGridTuple>,
    assets: Res<GameAssets>,
) {
    for (children, renderable_grid) in query_top.iter() {
        for &child in children.iter() {
            if let Ok((layer_children, name)) = query_1st_children_layer.get(child) {
                if *name != Name::new("Sprites") {
                    continue;
                }
                //~

                for &layer_child in layer_children.iter() {
                    if let Ok((mut sprite, mut t, mut texture, coord, pivot)) =
                        query.get_mut(layer_child)
                    {
                        let num =
                            renderable_grid.get_render_id(coord.r, coord.c, Some(&query_tool));
                        let info = renderable_grid.get_sprite_info(num, &assets);

                        sprite.color = info.color;
                        if pivot.is_some() {
                            sprite.color = Color::YELLOW;
                        }
                        *texture = info.image;
                        t.translation.z = info.z;
                    }
                }
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
            sprite.color =
                if let Some(element) = field_state.get_element(IVec2::new(coord.c, coord.r)) {
                    match element.kind {
                        FieldElementKind::Block(_) => Color::RED,
                        _ => Color::WHITE,
                    }
                } else {
                    Color::WHITE
                }
        }
    }
}
