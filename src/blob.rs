use bevy::{ecs::{system::EntityCommands}, prelude::*, log};
use leafwing_input_manager::prelude::*;

use crate::{prelude::*};

// @todo seperate into position related and game-logic releated
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Blob {
    /// the pivot of the blob that defines the center
    pub pivot: IVec2,

    /// the movement direction that can be changed by tools
    pub movement: IVec2,

    /// information if the blob is active (receives movement updates, or not) (for pause after cutting)
    pub active: bool,

    /// a list of blocks that belong to this blob
    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub blocks: Vec<Entity>,

    /// a flag indicting of the blob has already been teleported to the production field
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
    pub fn new(position: IVec2) -> Self {
        Blob {
            pivot: position,
            movement: IVec2::new(0, 1),
            active: true,
            blocks: vec![],
            transferred: false,
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

    pub fn rotate_left<'a>(
        &mut self, block_iter: 
        &mut impl Iterator<Item = Mut<'a, Block>>,
        ev_view: &mut EventWriter<ViewUpdate>,
        id: Entity,
    ) {
        for mut block in block_iter.filter(|b| b.blob.is_some() && b.blob.unwrap() == id) {
            /*
            block.relative_position = block.relative_position.map(|rp| IVec2::new(rp.y, -rp.x));
            */
            let old_pos = block.relative_position.unwrap_or_default();
            block.relative_position = Some(rotate_coord(block.relative_position.unwrap(), Rotation::Left));
            block.position = block.relative_position.unwrap_or_default() + self.pivot;

            log::info!("Rotated from {} to {}", old_pos, block.relative_position.unwrap_or_default());
        }

        ev_view.send(ViewUpdate::BlobRotated(id, Rotation::Left))
    }

    pub fn rotate_right<'a>(
        &mut self, 
        block_iter: impl Iterator<Item = Mut<'a, Block>>, 
        ev_view: &mut EventWriter<ViewUpdate>, 
        id: Entity,
    ) {
        for mut block in block_iter.filter(|b| b.blob.is_some() && b.blob.unwrap() ==id) {
            block.relative_position = block.relative_position.map(|rp| IVec2::new(-rp.y, rp.x));
            block.position = block.relative_position.unwrap_or_default() + self.pivot;
        }

        ev_view.send(ViewUpdate::BlobRotated(id, Rotation::Right))
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
    body: BlobBody,
    name: &str,
    field: Entity,
    position: IVec2, // @todo later work with coordinates and parent tetris-field
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let mut blob = Blob::new(position);

    // use commands to generate blob entity and block entities
    let blob_id = {
        let id = commands.spawn_bundle(SpatialBundle {
        ..Default::default()
        })
        .id();
    
        blob.blocks = Block::spawn_blocks_of_blob(commands, &body, &blob, id, field);

        id
    };

    // use commands to adapt the blob entity
    let mut ec = commands.entity(blob_id);
    ec.insert(blob)
        .insert(Name::new(name.to_string()));
    adapter(&mut ec);

    blob_id
}

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_blob_by_player(
    mut query: Query<(&ActionState<TetrisActionsWASD>, &mut Blob, Entity)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut ev_view: EventWriter<ViewUpdate>,
    turn: Res<Turn>,
) {
    // continue here
    // check if we are in a turn change...
    if turn.is_new_turn() {
        query.for_each_mut(|(s, mut blob, blob_id)| {
            
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
                    delta, 
                    block_iter, 
                    Some(&mut ev_view));
            }
        });
    }
}
