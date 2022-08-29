use crate::prelude::*;
use bevy::{log, prelude::*};

pub struct BlobMoveEvent {
    delta: (i32, i32),

    entity: Entity,
}

pub struct BlobTeleportEvent {
    entity: Entity,
}

pub fn generate_move_events_by_gravity(
    query_collector: Query<(Entity, &Blob)>,
    mut query: Query<(Entity, &Blob)>,
    turn: Res<Turn>,
    mut ev: EventWriter<BlobMoveEvent>,
) {
    if turn.is_new_turn() {
        let mut vec: Vec<(Entity, i32)> = query_collector
            .iter()
            .map(|(entity, blob)| (entity, blob.coordinate.y))
            .collect();

        vec.sort_by(|left, right| left.1.cmp(&right.1));

        for (id, _) in vec {
            if let Ok((e, blob)) = query.get_mut(id) {
                if blob.active {
                    // events are afterwards read in order!
                    ev.send(BlobMoveEvent {
                        delta: (blob.movement.x, blob.movement.y),
                        entity: e,
                    });
                }
            }
        }
    }
}

pub fn move_factory_blobs_by_events(
    mut query: Query<(&Parent, Entity, &mut Blob)>,
    parent_query: Query<(Entity, &Field), With<FactoryFieldTag>>,
    mut block_query: Query<(Entity, &mut Block)>,
    mut ev_move: EventReader<BlobMoveEvent>,
    mut ev_teleport: EventWriter<BlobTeleportEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    for ev in ev_move.iter() {
        if let Ok((p, blob_id, mut blob)) = query.get_mut(ev.entity) {
            let (entity, field) = parent_query.single();
            if entity != p.get() {
                continue;
            }
            //~

            if blob.active {
                let (tc, tr) = (blob.coordinate.x + ev.delta.0, blob.coordinate.y + ev.delta.1);
                let tv = IVec2::new(tc, tr);
                let state = field.get_field_state();

                if tr >= field.coordinate_limits().bottom {
                    ev_teleport.send(BlobTeleportEvent { entity: ev.entity });
                    continue;
                }
                //~

                if let Some(element) = state.get_element(tv) {
                    let mut do_move = false;
                    match element.kind {
                        // in the factory we can move out of region
                        FieldElementKind::Empty => {
                            do_move = true;
                        },
                        FieldElementKind::OutOfMovableRegion => {
                            do_move = !(tv.x < 0 || tv.x >= field.mov_size().0 as i32);
                        }
                        FieldElementKind::Tool(tool) => {
                            do_move = true;
                            match tool {
                                Tool::Move(d) => {
                                    blob.movement = d.into();
                                },
                                Tool::Rotate(d) => {
                                    let mut block_iter = block_query.iter_mut()
                                        .map(|(_, block)| block)
                                        .filter(|block| block.blob.is_some() && block.blob.unwrap() == blob_id );
                                    match d {
                                        RotateDirection::Left => blob.rotate_left(&mut block_iter),
                                        RotateDirection::Right => blob.rotate_right(&mut block_iter),
                                    }
                                },
                                Tool::Cutter(_) => {
                                    // @todo implement cutter tool
                                }
                                _ => {},
                            }
                        },
                        _ => {
                            bevy::log::warn!("Do nothing with target: {tc}, {tr} but stuck");
                        }
                    }
                    if do_move {
                        let block_iter = block_query.iter_mut()
                            .filter(|(_, block)| block.blob.is_some() && block.blob.unwrap() == blob_id );

                        move_blob(
                            (blob_id, &mut blob),
                            IVec2::new(ev.delta.0, ev.delta.1), 
                            block_iter, 
                            &mut ev_view);
                        
                    }
                }
            }
        }
    }
}

pub fn move_blob<'a>(
    (blob_id, blob): (Entity, &mut Blob),
    delta: IVec2,
    block_iter: impl Iterator<Item = (Entity, Mut<'a, Block>)>,
    ev_view: &mut EventWriter<ViewUpdate>,
) {
    blob.coordinate = blob.coordinate + delta;
    for (id, mut block) in block_iter {
        if let Some(blob) = block.blob  {
            if blob == blob_id {
                block.position += delta;
                ev_view.send(ViewUpdate::BlobMoved(id));
            }
        }
    }

    ev_view.send(ViewUpdate::BlobMoved(blob_id));
}

pub fn move_production_blobs_by_events(
    mut commands: Commands,
    mut query: Query<(Entity, &Parent, &mut Blob)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut parent_query: Query<&mut Field, With<ProductionFieldTag>>,
    mut ev: EventReader<BlobMoveEvent>,
    mut ev_view: EventWriter<ViewUpdate>,
) {
    for ev in ev.iter() {
        if let Ok((blob_id, p, mut blob)) = query.get_mut(ev.entity) {
            
            if let Ok(field) = parent_query.get_mut(p.get()) {
                let occ_coords: Vec<IVec2> = query_block.iter_mut()
                        .filter(|(_, block)| block.blob.is_some() && block.blob.unwrap() == blob_id)
                        .map(|(_, block)| block.position)
                        .collect(); 

                // transform grid
                let mut occ_coords_delta = occ_coords.clone();
                for ch in &mut occ_coords_delta {
                    ch.x += ev.delta.0;
                    ch.y += ev.delta.1;
                }

                let field_state = field.get_field_state();
                log::info!("Target by move:\n{:?}", occ_coords_delta);
                let occupied = field_state.is_any_coordinate_occupied(
                    &occ_coords_delta, 
                    Some(&occ_coords),
                    &|el| el.kind != FieldElementKind::Empty && el.kind != FieldElementKind::OutOfMovableRegion,
                );

                if !occupied {
                    let block_iter = query_block.iter_mut()
                        .filter(|(_, block)| block.blob.is_some() && block.blob.unwrap() == blob_id);
                    let delta = IVec2::new(ev.delta.0, ev.delta.1);
                    move_blob((blob_id, &mut blob), delta, block_iter, &mut ev_view);

                } else {
                    log::info!("Full Stop and occupy");
                    //blob.active = false;

                    //commands.entity(blob_id).despawn_recursive();
                }
            }
        }
    }
}

pub fn teleport_blob_out_of_factory(
    mut commands: Commands,
    query_fac: Query<Entity, With<FactoryFieldTag>>,
    query_prod: Query<Entity, With<ProductionFieldTag>>,
    mut query_blob: Query<(Entity, &Parent, &mut Blob)>,
    mut query_block: Query<(Entity, &mut Block)>,
    mut ev: EventReader<BlobTeleportEvent>,
    mut ev_view: EventWriter<ViewUpdate>
) {
    for ev in ev.iter() {
        if let Ok((blob_id, parent, blob)) = &mut query_blob.get_mut(ev.entity) {
            let fac = query_fac.single();
            let prod = query_prod.single();
            // we only teleport from the factory to the production
            if parent.get() != fac {
                continue;
            }
            //~ 

            let target = IVec2::new(blob.coordinate.x,-3);
            let delta = target - blob.coordinate;
            let block_iter = query_block.iter_mut()
                .filter(|(_, block)| block.blob.is_some() && block.blob.unwrap() == *blob_id);
            move_blob((*blob_id, blob), delta, block_iter, &mut ev_view);

            // @todo remove the hiearchy asap rendering is working
            commands.entity(fac).remove_children(&[ev.entity]);
            commands.entity(prod).push_children(&[ev.entity]);
            log::info!("Blob teleported!");
        }
    }
}
