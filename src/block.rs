use crate::prelude::*;
use bevy::{log, prelude::*};

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct PivotTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Reflect)]
pub struct OriginTag {}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect)]
pub struct Block {
    /// the position of the block in the coordinate system of the field
    pub position: IVec2,

    /// the relative position of the block from the parent grid body
    pub relative_position: Option<IVec2>,

    /// the group management entity of this block, if any, ignore in inspector, due to cycle
    #[cfg_attr(feature = "debug", inspectable(ignore))]
    pub group: Option<Entity>,

    /// the parent field of this block
    pub field: Entity,
}

impl Block {
    pub fn spawn_blocks_of_blob(
        commands: &mut Commands,
        body_def: &BodyDefinition,
        pivot: IVec2,
        group_id: Entity,
        field: Entity,
        handle_zero_position: bool,
    ) -> Vec<Entity> {
        let mut reval = vec![];
        for v in body_def.get_relative_positions() {
            if v == IVec2::ZERO && !handle_zero_position {
                continue;
            }
            //~

            let mut ec = commands.spawn_empty();
            if v == IVec2::ZERO {
                ec.insert(PivotTag {});
            }
            let id = ec
                .insert(Block {
                    position: pivot + v,
                    group: Some(group_id),
                    relative_position: Some(v),
                    field,
                })
                .insert(Name::new(format!("Block {},{}", v.x, v.y)))
                .id();

            log::info!("Spawn block {:?} at {v} with field {:?}", id, field);
            reval.push(id);
        }
        reval
    }
}

pub fn blocks_are_on_field<'a>(field_id: Entity, iter: impl Iterator<Item = &'a Block>) -> bool {
    let mut at_least_one = false;
    for block in iter {
        at_least_one = true;
        if block.field != field_id {
            return false;
        }
    }
    at_least_one
}
