use crate::prelude::*;
use bevy::{log, prelude::*, ecs::query::QueryIter};

pub struct BlobMoveEvent {
    delta: (i32, i32),

    entity: Entity,
}

pub struct BlobTeleportEvent {
    entity: Entity,
}

pub fn move_blobs_by_gravity(
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
    mut block_query: Query<&mut Block>,
    mut ev: EventReader<BlobMoveEvent>,
    mut ev_teleport: EventWriter<BlobTeleportEvent>,
    mut evt: EventWriter<ViewUpdate>,
) {
    for ev in ev.iter() {
        if let Ok((p, blob_id, mut blob)) = query.get_mut(ev.entity) {
            let (entity, field) = parent_query.single();
            if entity != p.get() {
                continue;
            }
            //~

            if blob.active {
                let (tc, tr) = (blob.coordinate.x + ev.delta.0, blob.coordinate.y + ev.delta.1);

                if tr >= field.coordinate_limits().bottom {
                    ev_teleport.send(BlobTeleportEvent { entity: ev.entity });
                    continue;
                }
                //~

                let (occupied, _) = field.is_coordinate_occupied(tc, tr, true);
                if occupied && !(tc < 0 || tr < 0) {

                    let num = field.occupied(field.coords_to_idx(tc as usize, tr as usize).unwrap());
                    if num.is_none() {
                        continue;
                    }
                    //~

                    let mut handled_by_tool = false;
                    let tool = TryInto::<Tool>::try_into(num.unwrap());
                    if let Ok(tool) = tool {
                        match tool {
                            Tool::Move(d) => {
                                blob.movement = d.into();
                                handled_by_tool = true;
                            }
                            Tool::Rotate(d) => {
                                let block_iter = block_query.iter_mut()
                                    .filter(|block| block.blob.is_some() && block.blob.unwrap() == blob_id );
                                match d {
                                    RotateDirection::Left => blob.rotate_left(block_iter),
                                    RotateDirection::Right => blob.rotate_right(block_iter),
                                }
                                handled_by_tool = true;
                            }
                            Tool::Cutter(_) => {
                                handled_by_tool = true;
                            }
                            _ => {}
                        }
                    }

                    if handled_by_tool {
                        blob.coordinate = IVec2::new(tc, tr);
                        evt.send(ViewUpdate::BlobMoved(blob_id));
                    }
                } else if !occupied {
                    blob.coordinate = IVec2::new(tc, tr);
                    evt.send(ViewUpdate::BlobMoved(blob_id));
                } else {
                    bevy::log::warn!("Do nothing with target: {tc}, {tr} but stuck");
                }
            }
        }
    }
}

pub fn move_production_blobs_by_events(
    mut commands: Commands,
    mut query: Query<(Entity, &Parent, &mut Blob)>,
    mut parent_query: Query<&mut Field, With<ProductionFieldTag>>,
    mut ev: EventReader<BlobMoveEvent>,
) {
    for ev in ev.iter() {
        if let Ok((id, p, mut blob)) = query.get_mut(ev.entity) {
            if let Ok(mut field) = parent_query.get_mut(p.get()) {
                let occ_coords: Vec<(i32, i32)> = blob
                    .occupied_coordinates()
                    .iter()
                    .map(|(x, y)| (x - pivot_coord().0 as i32, y - pivot_coord().1 as i32))
                    .collect();

                // transform grid
                let mut occ_coords_delta = occ_coords.clone();
                for ch in &mut occ_coords_delta {
                    ch.0 += ev.delta.0;
                    ch.1 += ev.delta.1;
                }

                // test for occupied
                let (occupied, _) = field.any_coordinates_occupied(&occ_coords_delta, true);

                if !occupied {
                    blob.coordinate.x += ev.delta.0;
                    blob.coordinate.y += ev.delta.1;
                } else {
                    log::info!("Full Stop and occupy");
                    blob.active = false;
                    field.occupy_coordinates(&occ_coords);

                    commands.entity(id).despawn_recursive();
                }
            }
        }
    }
}

pub fn teleport_blob_out_of_factory(
    mut commands: Commands,
    query_fac: Query<Entity, With<FactoryFieldTag>>,
    query_prod: Query<Entity, With<ProductionFieldTag>>,
    mut query_blob: Query<&mut Blob>,
    mut ev: EventReader<BlobTeleportEvent>,
) {
    for ev in ev.iter() {
        if let Ok(blob) = &mut query_blob.get_mut(ev.entity) {
            let fac = query_fac.single();
            let prod = query_prod.single();

            commands.entity(fac).remove_children(&[ev.entity]);
            blob.coordinate.y = -3;
            commands.entity(prod).push_children(&[ev.entity]);
        }
    }
}

pub fn move_field_content_down_if_not_occupied(
    mut query_field: Query<&mut Field>,
    turn: Res<Turn>,
) {
    if !turn.is_new_turn() {
        return;
    }
    for mut field in query_field.iter_mut() {
        if !field.tracks_occupied {
            continue;
        }
        //~

        field.move_down_if_possible();
    }
}
