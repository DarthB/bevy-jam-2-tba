use bevy::{ecs::system::EntityCommands, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{prelude::*};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Blob {
    pub coordinate: IVec2,

    pub movement: IVec2,

    pub active: bool,

    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub blocks: Vec<Entity>,

    pub texture: Handle<Image>,
}

pub struct BlobBody {
    pub block_positions: Vec<i32>,

    pub size: (usize, usize),

    pub pivot: (usize, usize),
}

impl BlobBody {
    pub fn new(positions: Vec<i32>) -> Self {
        BlobBody {
            block_positions: positions,
            size: (Blob::size(), Blob::size()),
            pivot: (4,4),
        }
    }
}

impl Blob {
    pub fn new(position: IVec2) -> Self {
        Blob {
            coordinate: position,
            movement: IVec2::new(0, 1),
            active: true,
            blocks: vec![],
            texture: Handle::default(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.movement.x == 0 && self.movement.y == 0
    }

    pub fn coords_to_idx(r: usize, c: usize) -> usize {
        coords_to_idx(r, c, Blob::size())
    }

    pub fn size() -> usize {
        9
    }

    pub fn empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn pivot_idx() -> usize {
        Blob::size().pow(2) / 2
    }

    pub fn rotate_left(&mut self) {
        unimplemented!()
    }

    pub fn rotate_right(&mut self) {
        unimplemented!()
    }

    /// the function calculates the occupied coordinates in the coordinate system of the
    /// parent (coordinate property)
    pub fn occupied_coordinates(&self) -> Vec<(i32, i32)> {
        unimplemented!();
    }
}

pub fn pivot_coord() -> (usize, usize) {
    (4, 4)
}

pub fn coords_to_idx(r: usize, c: usize, cs: usize) -> usize {
    r * cs + c
}

pub fn coords_to_px(x: i32, y: i32, rs: usize, cs: usize) -> (f32, f32) {
    (
        ((cs as f32 / -2.0) + x as f32) * PX_PER_TILE + PX_PER_TILE / 2.0,
        ((rs as f32 / 2.0) - y as f32) * PX_PER_TILE - PX_PER_TILE / 2.0,
    )
}

pub fn spawn_blob(
    commands: &mut Commands,
    texture: &Handle<Image>,
    assets: &GameAssets,
    body: BlobBody,
    name: &str,
    coord: IVec2, // @todo later work with coordinates and parent tetris-field
    use_old_rendering: bool,
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let mut blob = Blob::new(coord);
    
    // @todo replace with psi rendering
    blob.texture = texture.clone();

    let mut ec = commands.spawn_bundle(SpatialBundle {
        ..Default::default()
    });
    let id = ec.id();
    
    if !use_old_rendering {
        ec.insert(BlobExtra { blocks: blob.blocks.clone(), pivot: IVec2::ZERO, transferred: false });
    }

    ec.with_children(|cb| {
        if use_old_rendering {
            blob.spawn_render_entities(id, cb, assets);
        }
    })
    .insert(blob)
    .insert(Name::new(name.to_string()));
    adapter(&mut ec);

    // @todo use this over insert of BlobExtra
    Block::spawn_blocks_of_blob(commands, id, body);

    id
}

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_blob_by_player(
    mut query: Query<(&ActionState<TetrisActionsWASD>, &mut Blob)>, // get every entity, that has these three components
    turn: Res<Turn>, // get a bevy-internal resource that represents the time
) {
    // continue here
    // check if we are in a turn change...
    if turn.is_new_turn() {
        query.for_each_mut(|(s, mut blob)| {
            if s.pressed(TetrisActionsWASD::Up) {
                blob.coordinate.y -= 1;
            }

            if s.pressed(TetrisActionsWASD::Down) {
                blob.coordinate.y += 1;
            }

            if s.pressed(TetrisActionsWASD::Left) {
                blob.coordinate.x -= 1;
            }

            if s.pressed(TetrisActionsWASD::Right) {
                blob.coordinate.x += 1;
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

pub fn blob_update_transforms(
    mut query: Query<(&Blob, &mut Transform, &Parent)>,
    parent_query: Query<&Field>,
) {
    for (blob, mut transform, parent) in query.iter_mut() {
        if let Ok(field) = parent_query.get(parent.get()) {
            let (x, y) = field.coords_to_px(blob.coordinate.x, blob.coordinate.y);
            transform.translation = Vec3::new(x, y, transform.translation.z);
        }
    }
}
