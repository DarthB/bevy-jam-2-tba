use bevy::{prelude::*, ecs::system::EntityCommands};

use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Clone, Reflect)]
pub struct Field {
    movable_area_color: Color,

    edge_color: Color,

    movable_size: (usize, usize),

    additional_grids: UiRect<usize>,

    allow_overlap: UiRect<usize>,

    gravity: (i32, i32),
}

impl Field {
    pub fn as_factory() -> Self {
        Field { 
            movable_area_color: Color::MIDNIGHT_BLUE, 
            edge_color: Color::rgb(0.0, 0.2, 0.5),
            movable_size: (28,10),
            additional_grids: UiRect { left: 0, right: 1, top: 4, bottom: 4 }, 
            allow_overlap: UiRect { left: 10, right: 0, top: 4, bottom: 4 },
            gravity: (1,0),
        }
    }

    pub fn as_production_field() -> Self {
        Field {
            edge_color: Color::rgba(0.25,0.0,0.0, 1.0),
            ..Default::default()
         }
    }

    pub fn coords_to_px(&self, r: usize, c: usize) -> (f32, f32) {
        coords_to_px(r, c, self.movable_size.0, self.movable_size.1)
    }
}

impl Default for Field {
    fn default() -> Self {
        Self { 
            movable_area_color: Color::GRAY, 
            edge_color: Color::DARK_GRAY, 
            gravity: (0, -1),
            movable_size: (10, 20),
            additional_grids: UiRect { left: 1, right: 1, top: 0, bottom: 1 },
            allow_overlap: UiRect { left: 0, right: 0, top: 5, bottom: 0 },
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
    let mut ec = commands.spawn_bundle(SpatialBundle{
        transform: Transform { translation: trans, ..Default::default() },
        ..Default::default()
    });
    ec.with_children(|cb| {
        let sx = field.movable_size.0 + field.additional_grids.left + field.additional_grids.right;
        let sy = field.movable_size.1 + field.additional_grids.top + field.additional_grids.bottom;
        let f = PX_PER_TILE / 2.0;
        let ox = 
            field.additional_grids.right as f32 * f - 
            field.additional_grids.left as f32 * f;
        let oy = 
            field.additional_grids.top as f32 * f - 
            field.additional_grids.bottom as f32 * f;

        cb.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: field.edge_color,
                custom_size: Some(Vec2::new(
                    PX_PER_TILE * sx as f32,
                    PX_PER_TILE * sy as f32,
                )),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(ox,oy, Z_FIELD),
                ..Default::default()
            },
            ..Default::default()
        })
            .insert(Name::new("Background Sprite"))
        ;

        cb.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: field.movable_area_color,
                custom_size: Some(Vec2::new(
                    PX_PER_TILE * field.movable_size.0 as f32,
                    PX_PER_TILE * field.movable_size.1 as f32
                )),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, Z_FIELD+1.0),
                ..Default::default()
            },
            ..Default::default()
        })
            .insert(Name::new("Moveable Area Sprite"))
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
            .insert(Name::new("Pivot Sprite"))
            ;

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
                .insert(Name::new("Zero Sprite"))
                ;
        }

    })
        .insert(Name::new(name.to_string()))
        .insert(field)
    ;
    adapter(&mut ec);
    ec.id()
}