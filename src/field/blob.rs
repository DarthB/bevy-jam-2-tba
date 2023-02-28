//! The blob module consists of the main struct [`Blob`] that is used to control a solid blob,
//! e.g. a tetris stone on the field.
//!
//!
use bevy::{ecs::system::EntityCommands, log, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{
    bodies::BodyDefinition,
    input::TetrisActionsWASD,
    prelude::{move_blob, rotate_coord, Rotation, ViewUpdate},
    turn::Turn,
};

use super::prelude::*;

/// A component that represents a body on the grid. It supports rotation along it pivot
/// and provides a cutout function that can be used to cutout a [`Blob`] from anther [`Blob`].
/// Beside the definition of [`Blob`] it also gives [`super::Tool`] a shape on the [`Field`]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct GridBody {
    /// the pivot of the blob that defines the center
    pub pivot: IVec2,

    /// a list of blocks that belong to this blob
    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub blocks: Vec<Entity>,

    // todo move this flag somewhere else
    /// a flag indicting of the body has already been teleported to the production field
    pub transferred: bool,
}

impl GridBody {
    /// Generates a new GridBody at the specified position
    pub fn new(position: IVec2) -> Self {
        GridBody {
            pivot: position,
            blocks: vec![],
            transferred: false,
        }
    }

    pub fn get_relative_positions(
        &self,
        block_query: &mut Query<(Entity, &mut Block)>,
    ) -> Vec<IVec2> {
        let mut reval = Vec::with_capacity(self.blocks.len());
        // calculate new relative position for blocks
        for block_id in &self.blocks {
            if let Ok(block) = block_query.get(*block_id) {
                reval.push(block.1.relative_position.unwrap());
            }
        }
        reval
    }

    /// Performs a Cutout operations by forming a second blob from the given entities.
    ///
    /// # Arguments
    /// * `cutout_blocks` - A vector containing all the entity-ids of the [`Block`]s that form the second blob via cutout
    /// * `new_pivot` - The field space coordinate of pivot of the second blob.
    ///
    /// The remaining arguments are bevy internals to send spawn, send events and query for individual [`Block`] components
    pub fn cutout(
        &mut self,
        cutout_blocks: &Vec<Entity>,
        new_pivot: IVec2,
        commands: &mut Commands,
        ev_view: &mut EventWriter<ViewUpdate>,
        block_query: &mut Query<&mut Block>,
    ) {
        // todo: what happens when entities argument contains ids that are not part of self.blocks?

        // remove blocks from self and spawn new blob
        self.blocks.retain(|el| !cutout_blocks.contains(el));
        let new_blob_id = spawn_blob_from_cutout(commands, new_pivot, cutout_blocks);

        // calculate new relative position for blocks
        for block_id in cutout_blocks {
            if let Ok(mut block) = block_query.get_mut(*block_id) {
                block.group = Some(new_blob_id);
                block.relative_position = Some(block.position - new_pivot);
            }
        }

        // inform renderer
        ev_view.send(ViewUpdate::BlobCutout(new_blob_id));
    }

    /// the size is 9x9 fields - we decided for that magic number in one of the early meetings
    pub fn size() -> usize {
        9
    }

    pub fn coords_to_idx(r: usize, c: usize) -> usize {
        r * GridBody::size() + c
    }

    /// Rotates the blob left (counter-clock wise)
    pub fn rotate_left<'a>(
        &mut self,
        block_iter: &mut impl Iterator<Item = Mut<'a, Block>>,
        ev_view: &mut EventWriter<ViewUpdate>,
        id: Entity,
    ) {
        for mut block in block_iter.filter(|b| b.group.is_some() && b.group.unwrap() == id) {
            /*
            block.relative_position = block.relative_position.map(|rp| IVec2::new(rp.y, -rp.x));
            */
            let old_pos = block.relative_position.unwrap_or_default();
            block.relative_position = Some(rotate_coord(
                block.relative_position.unwrap(),
                Rotation::Left,
            ));
            block.position = block.relative_position.unwrap_or_default() + self.pivot;

            log::info!(
                "Rotated from {} to {}",
                old_pos,
                block.relative_position.unwrap_or_default()
            );
        }

        ev_view.send(ViewUpdate::BlobRotated(id, Rotation::Left))
    }

    /// Rotates the blob right (clock wise)
    pub fn rotate_right<'a>(
        &mut self,
        block_iter: impl Iterator<Item = Mut<'a, Block>>,
        ev_view: &mut EventWriter<ViewUpdate>,
        id: Entity,
    ) {
        for mut block in block_iter.filter(|b| b.group.is_some() && b.group.unwrap() == id) {
            block.relative_position = block.relative_position.map(|rp| IVec2::new(-rp.y, rp.x));
            block.position = block.relative_position.unwrap_or_default() + self.pivot;
        }

        ev_view.send(ViewUpdate::BlobRotated(id, Rotation::Right))
    }
}

/// A blob is a connection of blocks that together form a movable stone
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Blob {
    /// the movement direction that can be changed by tools
    pub movement: IVec2,

    /// information if the blob is active (receives movement updates, or not) (for pause after cutting)
    pub active: bool,

    /// a cutout blob is not part of the game anymore but only visual, we do want to get rid of it
    pub cutout: bool,
}

impl Blob {
    pub fn new_main() -> Self {
        Blob {
            movement: IVec2::new(0, 1),
            active: true,
            cutout: false,
        }
    }

    pub fn new_cutout() -> Self {
        Blob {
            movement: IVec2::new(0, 1),
            active: true,
            cutout: true,
        }
    }
}

pub fn spawn_blob_from_cutout(
    commands: &mut Commands,
    position: IVec2,
    blocks: &[Entity],
) -> Entity {
    commands
        .spawn(SpatialBundle::default())
        .insert(GridBody {
            pivot: position,
            blocks: blocks.to_owned(),
            transferred: false,
        })
        .insert(Blob::new_cutout())
        .insert(Name::new("Cutout-Blob"))
        .id()
}

pub fn spawn_blob_from_body_definition(
    commands: &mut Commands,
    body: BodyDefinition,
    name: &str,
    field: Entity,
    position: IVec2, // @todo later work with coordinates and parent tetris-field
    adapter: &dyn Fn(&mut EntityCommands),
) -> Entity {
    let mut grid_body = GridBody::new(position);
    // use commands to generate blob entity and block entities
    let blob_id = {
        let id = commands
            .spawn(SpatialBundle {
                ..Default::default()
            })
            .id();

        grid_body.blocks = Block::spawn_blocks_of_blob(commands, &body, position, id, field, true);

        id
    };

    // use commands to adapt the blob entity
    let mut ec = commands.entity(blob_id);
    ec.insert(Blob::new_main())
        .insert(grid_body)
        .insert(Name::new(name.to_string()));
    adapter(&mut ec);

    blob_id
}

/// Example of system that maps actions to movements on a controlled entity:
pub fn move_blob_by_player(
    mut query: Query<(&ActionState<TetrisActionsWASD>, &mut GridBody, Entity)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut ev_view: EventWriter<ViewUpdate>,
    turn: Res<Turn>,
) {
    // continue here
    // check if we are in a turn change...
    if turn.is_new_turn() {
        query.for_each_mut(|(s, mut body, blob_id)| {
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
                let mut block_iter = query_block
                    .iter_mut()
                    .map(|(_, block)| block)
                    .filter(|block| block.group == Some(blob_id));
                if s.pressed(TetrisActionsWASD::LRotate) {
                    body.rotate_left(&mut block_iter, &mut ev_view, blob_id);
                } else if s.pressed(TetrisActionsWASD::RRotate) {
                    body.rotate_right(&mut block_iter, &mut ev_view, blob_id);
                }
            }

            let block_iter = query_block
                .iter_mut()
                .filter(|(_, block)| block.group == Some(blob_id));

            if delta != IVec2::ZERO {
                move_blob(blob_id, &mut body, delta, block_iter, Some(&mut ev_view));
            }
        });
    }
}
