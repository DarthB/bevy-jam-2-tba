use bevy::{prelude::*, log};
use crate::prelude::*;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct PivotTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct OriginTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct Block {
    pub position: IVec2,

    pub blob: Option<Entity>,

    pub field: Option<Entity>,
}

impl Block {
    pub fn spawn_blocks_of_blob(
        commands: &mut Commands, 
        body: &BlobBody, 
        blob: &Blob,
        field: Entity,
    ) -> Vec<Entity> {
        let mut reval = vec![];
        for v in body.get_relative_positions() {
            let mut ec = commands.spawn();
            if v == IVec2::ZERO {
                ec.insert(PivotTag{});
            }
            let id = ec.insert(Block{ 
                    position: blob.coordinate + v, 
                    blob: None,
                    field: Some(field),
                })
                .insert(BlockExtra {
                    coordinate: v,
                })
                .insert(Name::new(format!("Block {},{}", v.x, v.y)))
                .id();

            log::info!("Spawn block {:?} at {v} with field {:?}", id, field);
            reval.push(id);
        }
        reval
    }
}

pub fn blocks_are_on_field<'a>(
    field_id: Entity, 
    iter: impl Iterator<Item = &'a Block>,
) -> bool {
    let mut at_least_one = false;
    for block in iter {
        at_least_one = true;
        if !block.field.is_some() || block.field.unwrap() != field_id {
            return false
        }
    }
    at_least_one
}