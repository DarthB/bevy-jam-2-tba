use bevy::prelude::*;
use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Block {
    pub position: IVec2,

    pub blob: Option<Entity>,
}

impl Block {
    pub fn spawn_blocks_of_blob(commands: &mut Commands, blob_id: Entity, body: BlobBody) {
        for idx in 0..body.block_positions.len() {
            let num = body.block_positions[idx];
            if num == 0 {continue;}
            //~

            let x = (num % body.size.0 as i32) - body.pivot.0 as i32;
            let y = (num / body.size.1 as i32) - body.pivot.1 as i32;

            commands.spawn().insert(
                Block{ position: IVec2 { x, y }, blob: Some(blob_id) }
            );
        }
    }
}