use bevy::prelude::*;
use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct PivotTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Block {
    pub position: IVec2,

    pub blob: Option<Entity>,

    pub field: Option<Entity>,
}

impl Block {
    pub fn spawn_blocks_of_blob(commands: &mut Commands, body: &BlobBody, blob: &Blob) -> Vec<Entity> {
        let mut reval = vec![];
        for v in body.get_relative_positions() {
            let mut ec = commands.spawn();
            if v == IVec2::ZERO {
                ec.insert(PivotTag{});
            }
            let id = ec.insert(Block{ 
                    position: blob.coordinate + v, 
                    blob: None,
                    field: None,
                })
                .insert(Name::new(format!("Block {},{}", v.x, v.y)))
                .id();

            reval.push(id);
        }
        reval
    }
}