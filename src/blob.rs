use bevy::{ecs::system::EntityCommands, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Blob {
    pub body: Vec<i32>,
}

/// used for tagging the sprites that belong to a Blob
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct BlobGridSprite {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Coordinate {
    pub r: usize,
    pub c: usize,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct BlobGravity {
    pub gravity: i32,
}

pub fn gen_l_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

pub fn gen_t_body() -> Vec<i32> {
    vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 1, 0, 0, 0, 0, //
        0, 0, 0, 1, 1, 1, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
        0, 0, 0, 0, 0, 0, 0, 0, 0, //
    ]
}

impl Blob {
    pub fn new(body: Vec<i32>) -> Self {
        Blob { body }
    }

    pub fn coords_to_px(r: usize, c: usize) -> (f32, f32) {
        coords_to_px(r, c, Blob::size(), Blob::size())
    }

    pub fn coords_to_idx(r: usize, c: usize) -> usize {
        coords_to_idx(r, c, Blob::size())
    }

    pub fn rotate_left(&mut self) {
        let mut rot_vec = vec![0; Blob::size().pow(2)];

        for r in 0..Blob::size() - 1 {
            for c in 0..Blob::size() - 1 {
                let index = Blob::coords_to_idx(r, c);
                let index_in_new = (Blob::size() - c) * Blob::size() - (Blob::size() - r);
                rot_vec[index_in_new] = self.body[index];
            }
        }

        self.body = rot_vec;
    }

    pub fn rotate_right(&mut self) {
        let mut rot_vec = vec![0; Blob::size().pow(2)];

        for r in 0..Blob::size() - 1 {
            for c in 0..Blob::size() - 1 {
                let index = Blob::coords_to_idx(r, c);
                let index_in_new = Blob::size() - r - 1 + c * Blob::size();
                rot_vec[index_in_new] = self.body[index];
            }
        }

        self.body = rot_vec;
    }

    pub fn size() -> usize {
        9
    }
}

pub fn pivot_idx() -> usize {
    Blob::size().pow(2) / 2
}

pub fn pivot_coord() -> (usize, usize) {
    (4, 4)
}

pub fn coords_to_idx(r: usize, c: usize, cs: usize) -> usize {
    r * cs + c
}

pub fn coords_to_px(r: usize, c: usize, rs: usize, cs: usize) -> (f32, f32) {
    (
        ((cs as f32 / -2.0) + c as f32) * PX_PER_TILE + PX_PER_TILE / 2.0,
        ((rs as f32 / 2.0) - r as f32) * PX_PER_TILE - PX_PER_TILE / 2.0,
    )
}

pub fn blob_sprite_color_and_zorder(num: i32) -> (Color, f32) {
    if num == 1 {
        (Color::BLACK, Z_SOLID)
    } else {
        (Color::rgba(0.5, 0.5, 0.5, 0.25), Z_TRANS)
    }
}

pub fn spawn_blob(
    commands: &mut Commands,
    body: Vec<i32>,
    name: &str,
    trans: Vec3, // @todo later work with coordinates and parent tetris-field
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let blob = Blob::new(body);

    let mut ec = commands.spawn_bundle(SpatialBundle {
        transform: Transform {
            translation: trans,
            ..Default::default()
        },
        ..Default::default()
    });
    ec.insert(BlobGravity { gravity: 0 })
        .with_children(|cb| {
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
                let (x, y) = Blob::coords_to_px(0, 0);
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
                .insert(Name::new("ZERO Sprite"));
            }

            for r in 0..Blob::size() {
                for c in 0..Blob::size() {
                    let (color, z) = blob_sprite_color_and_zorder(blob.body[r * Blob::size() + c]);
                    let (x, y) = Blob::coords_to_px(r, c);

                    cb.spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::ONE * PX_PER_TILE - 2.0),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: Vec3::new(x, y, z),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Coordinate { r, c })
                    .insert(Name::new(format!("grid {}:{}", r, c)));
                }
            }
        })
        .insert(blob)
        .insert(Name::new(name.to_string()));
    adapter(&mut ec);
    ec.id()
}

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_blob_by_player(
    mut query: Query<(&ActionState<TetrisActionsWASD>, &mut Blob, &mut Transform)>, // get every entity, that has these three components
    turn: Res<Turn>, // get a bevy-internal resource that represents the time
) {
    // continue here
    // check if we are in a turn change...
    if turn.is_new_turn() {
        query.for_each_mut(|(s, mut blob, mut t)| {
            if s.pressed(TetrisActionsWASD::Up) {
                t.translation.y += PX_PER_TILE;
            }

            if s.pressed(TetrisActionsWASD::Down) {
                t.translation.y -= PX_PER_TILE;
            }

            if s.pressed(TetrisActionsWASD::Left) {
                t.translation.x -= PX_PER_TILE;
            }

            if s.pressed(TetrisActionsWASD::Right) {
                t.translation.x += PX_PER_TILE;
            }

            if s.pressed(TetrisActionsWASD::LRotate) {
                blob.rotate_left();
            }

            if s.pressed(TetrisActionsWASD::RRotate) {
                blob.rotate_right();
            }
        });
    }
}

pub fn blob_update_sprites(
    query: Query<(&Blob, &Children)>,
    mut q_children: Query<(&mut Sprite, &mut Transform, &Coordinate)>,
) {
    for (blob, children) in query.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut t, coord)) = q_children.get_mut(child) {
                let (color, z) = blob_sprite_color_and_zorder(
                    blob.body[coords_to_idx(coord.r, coord.c, Blob::size())],
                );
                sprite.color = color;
                t.translation.z = z;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::blob::pivot_coord;

    use super::{pivot_idx, Blob};

    pub fn gen_3x3_test_body() -> Vec<i32> {
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 1, 2, 3, 0, 0, 0, //
            0, 0, 0, 4, 5, 6, 0, 0, 0, //
            0, 0, 0, 7, 8, 9, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]
    }

    pub fn gen_3x3_lr_test_body() -> Vec<i32> {
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 3, 6, 9, 0, 0, 0, //
            0, 0, 0, 2, 5, 8, 0, 0, 0, //
            0, 0, 0, 1, 4, 7, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]
    }

    pub fn gen_3x3_rr_test_body() -> Vec<i32> {
        vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 7, 4, 1, 0, 0, 0, //
            0, 0, 0, 8, 5, 2, 0, 0, 0, //
            0, 0, 0, 9, 6, 3, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]
    }

    #[test]
    fn test_rotation_left() {
        let mut blob = Blob::new(gen_3x3_test_body());
        blob.rotate_left();
        assert_eq!(blob.body, gen_3x3_lr_test_body());
    }

    #[test]
    fn test_rotation_right() {
        let mut blob = Blob::new(gen_3x3_test_body());
        blob.rotate_right();
        assert_eq!(blob.body, gen_3x3_rr_test_body());
    }

    #[test]
    fn test_pivot() {
        let blob = Blob::new(gen_3x3_test_body());
        assert_eq!(blob.body[pivot_idx()], 5);

        let (r, c) = pivot_coord();
        assert_eq!(blob.body[Blob::coords_to_idx(r, c)], 5);
    }
}