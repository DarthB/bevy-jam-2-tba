use bevy::{ecs::{system::EntityCommands}, prelude::*, log};
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

    /// this is a hack
    pub relative_positions: Vec<IVec2>,

    pub texture: Handle<Image>,

    pub transferred: bool,
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Reflect)]
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

    pub fn get_relative_positions(&self) -> Vec<IVec2> {
        let mut reval = vec![];

        for idx in 0..self.block_positions.len() {
            let num = self.block_positions[idx];
            if num == 0 {continue;}
            //~

            let x = (idx as i32 % self.size.0 as i32) - self.pivot.0 as i32;
            let y = (idx as i32 / self.size.1 as i32) - self.pivot.1 as i32;
            
            reval.push(IVec2 {x, y});
        }

        reval
    }
}

impl Blob {
    pub fn new(position: IVec2, body: &BlobBody) -> Self {
        Blob {
            coordinate: position,
            movement: IVec2::new(0, 1),
            active: true,
            blocks: vec![],
            texture: Handle::default(),
            transferred: false,
            relative_positions: body.get_relative_positions(),
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

    pub fn rotate_left<'a, I>(&mut self, block_iter: &mut I, ev_view: &mut EventWriter<ViewUpdate>, id: Entity)
        where I: Iterator<Item = Mut<'a, Block>> {
            self.relative_positions.clear();
        for mut block in block_iter {
            block.position -= self.coordinate;
            block.position = IVec2::new(block.position.y, -block.position.x);
            self.relative_positions.push(block.position);
            block.position += self.coordinate;
        }

        ev_view.send(ViewUpdate::BlobRotated(id, Rotation::Left))
    }

    pub fn rotate_right<'a>(&mut self, block_iter: impl Iterator<Item = Mut<'a, Block>>, ev_view: &mut EventWriter<ViewUpdate>, id: Entity) {
        self.relative_positions.clear();
        for mut block in block_iter {
            block.position -= self.coordinate;
            block.position = IVec2::new(-block.position.y, block.position.x);
            self.relative_positions.push(block.position);
            block.position += self.coordinate;
        }

        ev_view.send(ViewUpdate::BlobRotated(id, Rotation::Right))
    }

    /// the function calculates the occupied coordinates in the coordinate system of the
    /// parent (coordinate property)
    pub fn coordinates_of_blocks(&self, relative: bool) -> Vec<IVec2> {
        if relative {
            self.relative_positions.clone()
        } else {
            self.relative_positions.iter()
                .map(|v| *v + self.coordinate)
                .collect()

            }
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
    body: BlobBody,
    name: &str,
    field: Entity,
    position: IVec2, // @todo later work with coordinates and parent tetris-field
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let mut blob = Blob::new(position, &body);
    
    // @todo replace with psi rendering
    blob.texture = texture.clone();

    // use commands to generate blob entity and block entities
    let blob_id = {
        let id = commands.spawn_bundle(SpatialBundle {
        ..Default::default()
        })
        .id();
    
        // @todo borow checker and getting the ids into each other
        blob.blocks = Block::spawn_blocks_of_blob(commands, &body, &blob, id, field);

        id
    };

    // use commands to adapt the blob entity
    let mut ec = commands.entity(blob_id);
    // @todo remove extra stuff
    ec.insert(BlobExtra { blocks: blob.blocks.clone(), pivot: IVec2::ZERO, transferred: false })
        .insert(blob)
        .insert(Name::new(name.to_string()));
    adapter(&mut ec);

    blob_id
}

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_blob_by_player(
    mut query: Query<(&ActionState<TetrisActionsWASD>, &mut Blob, &mut BlobExtra, Entity)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut ev_view: EventWriter<ViewUpdate>,
    turn: Res<Turn>,
) {
    // continue here
    // check if we are in a turn change...
    if turn.is_new_turn() {
        query.for_each_mut(|(s, mut blob, mut extra, blob_id)| {
            
            let mut delta = IVec2::ZERO;
            if s.pressed(TetrisActionsWASD::Up) {
                delta.y -= 1;
            }

            if s.pressed(TetrisActionsWASD::Down) {
                delta.y += 1;
            }

            if s.pressed(TetrisActionsWASD::Left) {
                delta.x -= 1;
            }

            if s.pressed(TetrisActionsWASD::Right) {
                delta.x += 1;
            }
        
            {
                let mut block_iter = query_block.iter_mut()
                    .map(|(_,block)| block)
                    .filter(|block| block.blob.is_some() && block.blob.unwrap() == blob_id);
                if s.pressed(TetrisActionsWASD::LRotate) {
                    blob.rotate_left(&mut block_iter, &mut ev_view, blob_id);
                } else if s.pressed(TetrisActionsWASD::RRotate) {
                    blob.rotate_right(&mut block_iter, &mut ev_view, blob_id);
                }
            }

            let block_iter = query_block.iter_mut()
                .filter(|(_,block)| block.blob.is_some() && block.blob.unwrap() == blob_id);

            if delta != IVec2::ZERO {
                move_blob(
                    blob_id, 
                    &mut blob,
                    &mut extra,
                    delta, 
                    block_iter, 
                    Some(&mut ev_view));
            }
        });
    }
}
